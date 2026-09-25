//! `service` — Persistent background service running alongside the app for
//! handling anything that needs a background worker:
//! - Download link fetching and token resolution
//! - Direct file streaming with byte progress
//! - Direct link extraction for supported hosts (Pixeldrain, Mediafire, Fileknot, etc.)
//! - Downloads folder archive auto-ingestion & extraction into the library
//! - Live progress & status event emission to the Tauri webview

use crate::core::folder;
use crate::core::library;
use crate::core::queue::{Queue, QueueJob, Status};
use crate::core::{Context, Error};
use crate::resolver;
use std::fs;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// Direct streamable hosts or hosts with known direct-link extraction.
pub const DIRECT_STREAM_HOSTS: &[&str] = &["fileknot", "pixeldrain", "mediafire", "workupload"];

/// Archive extensions we treat as downloadable game archives.
pub const ARCHIVE_EXTS: &[&str] = &[
    ".zip", ".7z", ".rar", ".exe", ".tar.gz", ".tar.bz2", ".tar.xz",
];

/// Check if a host slug is direct-streamable or extractable.
pub fn is_direct_host(host: &str) -> bool {
    let lower = host.trim().to_lowercase();
    DIRECT_STREAM_HOSTS
        .iter()
        .any(|h| h.eq_ignore_ascii_case(&lower))
}

/// Try to extract a direct download URL from an external host URL.
/// E.g. pixeldrain `/u/{id}` -> `/api/file/{id}`
/// Mediafire landing page -> parses `<a id="downloadButton" href="...">`
pub fn extract_direct_url(host: &str, url: &str) -> Option<String> {
    let lower_host = host.trim().to_lowercase();
    if lower_host == "pixeldrain" {
        // Pixeldrain URL: https://pixeldrain.com/u/<id>
        if let Some(idx) = url.find("/u/") {
            let id = &url[idx + 3..];
            let clean_id = id.split(['?', '/', '#']).next().unwrap_or(id);
            if !clean_id.is_empty() {
                return Some(format!("https://pixeldrain.com/api/file/{clean_id}"));
            }
        }
    } else if lower_host == "mediafire" {
        // Mediafire page scraping: fetch page and extract downloadButton href
        if let Ok(html) = crate::scraper::fetch(url) {
            if let Some(pos) = html.find("id=\"downloadButton\"") {
                let window = &html[pos.saturating_sub(200)..std::cmp::min(pos + 300, html.len())];
                if let Some(href_idx) = window.find("href=\"") {
                    let rest = &window[href_idx + 6..];
                    if let Some(end_quote) = rest.find('"') {
                        let link = &rest[..end_quote];
                        if link.starts_with("http") {
                            return Some(link.to_string());
                        }
                    }
                }
            }
        }
    } else if lower_host == "fileknot" {
        return Some(url.to_string());
    }

    // Check if the URL itself directly ends in an archive extension
    let clean_url = url.split('?').next().unwrap_or(url).to_lowercase();
    if ARCHIVE_EXTS.iter().any(|ext| clean_url.ends_with(ext)) {
        return Some(url.to_string());
    }

    None
}

/// Event payload emitted to Tauri frontend when a job updates.
#[derive(Debug, Clone, serde::Serialize)]
pub struct BackgroundJobEvent {
    pub id: u64,
    pub slug: String,
    pub version: String,
    pub platform: String,
    pub tab: String,
    pub status: String,
    pub message: Option<String>,
    pub bytes_done: u64,
    pub bytes_total: u64,
}

impl From<&QueueJob> for BackgroundJobEvent {
    fn from(j: &QueueJob) -> Self {
        Self {
            id: j.id,
            slug: j.slug.clone(),
            version: j.version.clone(),
            platform: j.platform.clone(),
            tab: j.tab.clone(),
            status: j.status.as_str().to_string(),
            message: j.message.clone(),
            bytes_done: j.bytes_done,
            bytes_total: j.bytes_total,
        }
    }
}

/// Scan the download root for completed archives (e.g. downloaded from browser)
/// and auto-enqueue them for extraction into `lzapps/<slug>/`.
pub fn ingest_completed_archives(ctx: &Context, queue: &Queue) -> Result<usize, Error> {
    let download_root = match folder::download_root(ctx) {
        Ok(p) => p,
        Err(_) => return Ok(0),
    };
    if !download_root.exists() {
        return Ok(0);
    }

    let lzapps = match folder::lzapps_root(ctx) {
        Ok(p) => p,
        Err(_) => return Ok(0),
    };

    let mut queued = 0;
    let mut stack = vec![download_root];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }

            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            // Skip partial/incomplete browser downloads
            if name.ends_with(".crdownload") || name.ends_with(".part") || name.ends_with(".tmp") {
                continue;
            }

            if !library::is_supported_archive(&path) {
                continue;
            }

            // Parse clean title, version, and platform
            let (title, version, platform) = library::parse_archive_filename(name);
            let slug = crate::core::game_arg_slug(&title);

            // Check if already extracted or queued
            let target_dir = lzapps.join(&slug);
            if target_dir.join("app.json").exists() {
                continue;
            }

            let path_str = path.to_string_lossy().to_string();
            let target_str = target_dir.to_string_lossy().to_string();

            // Check if this file is already in the queue
            if queue.snapshot().iter().any(|j| {
                j.source.as_deref() == Some(&path_str)
                    && (j.status == Status::Queued
                        || j.status == Status::Extracting
                        || j.status == Status::Completed)
            }) {
                continue;
            }

            let _ = queue.enqueue_extract(slug, version, platform, path_str, target_str);
            queued += 1;
        }
    }

    Ok(queued)
}

/// Spawns the dedicated, long-running background service alongside the app.
pub fn start_service(
    queue: Arc<Queue>,
    ctx: Context,
    app_handle: Option<tauri::AppHandle>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut last_scan = SystemTime::now();

        loop {
            // Check if there is a queued job; wait with a 5s timeout so we can also
            // periodically do folder ingestion and health checks.
            let next_id = queue.find_queued_or_timeout(Duration::from_secs(5));

            if let Some(id) = next_id {
                let grace = crate::core::scheduler::grace_for(&ctx);
                let app_for_process = app_handle.clone();
                let queue_for_emit = Arc::clone(&queue);

                crate::core::queue::process(
                    &queue,
                    &ctx,
                    id,
                    &mut |url| crate::scraper::fetch(url),
                    &mut |go| resolver::resolve(go),
                    &mut |job, progress| {
                        crate::core::download::dispatch_with(
                            job,
                            &mut crate::scraper::download_stream,
                            progress,
                        )
                    },
                    grace,
                    &mut |d| std::thread::sleep(d),
                );

                // Emit job state update to webview
                if let Some(ref app) = app_for_process {
                    use tauri::Emitter;
                    if let Some(job) = queue_for_emit.get(id) {
                        let _ = app.emit(
                            "background-service-job-update",
                            BackgroundJobEvent::from(&job),
                        );
                    }
                }
            }

            // Periodically check download folder for completed browser downloads (every 10s)
            if last_scan
                .elapsed()
                .map(|d| d >= Duration::from_secs(10))
                .unwrap_or(false)
            {
                last_scan = SystemTime::now();
                let _ = ingest_completed_archives(&ctx, &queue);
            }
        }
    })
}
