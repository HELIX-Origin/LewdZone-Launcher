//! `download` — resolve a token and download the file (or hand the resolved
//! URL to the OS). Runs inline in the CLI or through the async queue worker.
//!
//! The pipeline is: pick a `Game` page → find the `DownloadEntry` matching
//! (version, platform, tab) → resolve its go-link via the start→reveal API →
//! either stream the file into a canonical target (hosts whose reveal is a
//! direct file URL — see `DIRECT_STREAM_HOSTS`, folder-organizer) or pass the
//! URL to the OS default handler (installed cloud app / browser; zero config).
//! Streamed `.zip` archives are extracted into `<lzapps_root>/<slug>/` and
//! tracked with an itch.io-style `app.json` manifest.
//! The game fetch, the resolver, and the stream are injectable so every path
//! is covered offline with fixtures (Rule 11).

use std::io::{Read, Write};
use std::path::PathBuf;

use crate::core::extract;
use crate::core::folder;
use crate::core::models::{DownloadEntry, Game};
use crate::core::{Context, Error};
use crate::resolver;
use crate::scraper;

/// A fully dispatched download job (what `run` prints back).
#[derive(Debug, Clone, serde::Serialize)]
pub struct Job {
    pub game: String,
    pub title: String,
    pub post_id: Option<i64>,
    pub engine: Option<String>,
    pub version: String,
    pub platform: String,
    pub tab: String,
    pub url: String,
    pub target: Option<PathBuf>,
    pub install_dir: Option<PathBuf>,
}

/// The user-facing selection of what to fetch and grab from a game page.
#[derive(Debug, Clone, Default)]
pub struct Select<'a> {
    pub game: &'a str,
    pub version: &'a str,
    pub platform: &'a str,
    pub tab: &'a str,
    /// Restrict to one host slug (dropdown choice); `None` = all sources.
    pub source: Option<&'a str>,
}

/// Pick the version object matching `version` (or the site-flagged latest),
/// then collect every official/community entry for `platform`, ordered by the
/// user's `source-priority` (preferred hosts first, stable site order within a
/// rank), optionally restricted to a single chosen host (`source`).
///
/// Returns a user-facing error naming what was actually available so the
/// mismatch is diagnosable without guessing (dispatch-builder rule).
pub fn pick_entries<'a>(
    game: &'a Game,
    version: &str,
    platform: &str,
    tab: &str,
) -> Result<Vec<&'a DownloadEntry>, Error> {
    pick_entries_with(game, version, platform, tab, None, None)
}

/// `pick_entries` + `source-priority` ordering and optional host restriction.
pub fn pick_entries_with<'a>(
    game: &'a Game,
    version: &str,
    platform: &str,
    tab: &str,
    priority: Option<&str>,
    source: Option<&str>,
) -> Result<Vec<&'a DownloadEntry>, Error> {
    let ver = game
        .versions
        .iter()
        .find(|v| {
            version != "latest"
                && (v.label.eq_ignore_ascii_case(&format!("v{version}"))
                    || v.label.eq_ignore_ascii_case(version))
        })
        .or_else(|| {
            if version == "latest" {
                game.versions
                    .iter()
                    .find(|v| v.is_latest)
                    .or_else(|| game.versions.first())
            } else {
                None
            }
        })
        .ok_or_else(|| {
            let have: Vec<&str> = game.versions.iter().map(|v| v.label.as_str()).collect();
            Error::Usage(format!(
                "game '{}' has no version '{version}' (available: {})",
                game.slug,
                have.join(", ")
            ))
        })?;

    let pool = match tab {
        "official" => &ver.official,
        "community" => &ver.community,
        other => {
            return Err(Error::Usage(format!(
                "unknown tab '{other}' (expected 'official' or 'community')"
            )))
        }
    };
    let norm = crate::core::models::normalize_platform(platform);
    let mut entries: Vec<&DownloadEntry> = pool
        .iter()
        // Product policy: only live, reputable mirrors may be used (dead /
        // dodgy hosts are de-listed at the resolver). Never offer the rest.
        .filter(|e| e.platform.as_deref().map(|p| p == norm).unwrap_or(false))
        .filter(|e| crate::resolver::validate_host(&e.host).is_ok())
        .filter(|e| source.is_none_or(|s| e.host.eq_ignore_ascii_case(s)))
        .collect();

    if entries.is_empty() {
        let have: Vec<&str> = pool
            .iter()
            .filter_map(|e| e.platform.as_deref())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        return Err(Error::Usage(format!(
            "no {tab} entry for platform '{norm}' in {} (available: {})",
            ver.label,
            if have.is_empty() {
                "none".to_string()
            } else {
                have.join(", ")
            }
        )));
    }
    // Stable sort: preferred hosts first (earlier in the priority list wins),
    // site order preserved within a rank; never-listed hosts sort last.
    entries.sort_by_key(|e| {
        let rank = resolver::priority_rank(&e.host, priority);
        if rank == 0 {
            usize::MAX
        } else {
            rank
        }
    });
    Ok(entries)
}

/// One selectable source on a game's Download panel.
#[derive(Debug, Clone, serde::Serialize)]
pub struct HostSource {
    pub host: String,
    pub label: String,
    /// True when the `source-priority` preference names this host (and it's
    /// available for this game) — the dropdown's default.
    pub preferred: bool,
}

/// Compute the ordered list of sources (deduped hosts) available for a
/// selection, honoring the user's `source-priority`. The first listed host the
/// preference names is flagged `preferred`. Shared by the game page's source
/// dropdown and the CLI (`--source`).
pub fn sources_for(
    game: &Game,
    version: &str,
    platform: &str,
    tab: &str,
    priority: Option<&str>,
) -> Result<Vec<HostSource>, Error> {
    let entries = pick_entries_with(game, version, platform, tab, priority, None)?;
    let mut out: Vec<HostSource> = Vec::new();
    for e in entries {
        if out.iter().any(|s| s.host == e.host) {
            continue;
        }
        let preferred = crate::resolver::priority_rank(&e.host, priority) > 0
            && !out.iter().any(|s| s.preferred);
        out.push(HostSource {
            host: e.host.clone(),
            label: crate::core::native::app_name(&e.host).to_string(),
            preferred,
        });
    }
    Ok(out)
}

/// Fetch the game + settings and return `sources_for` for the GUI dropdown.
/// Mirrors `jobs_for`'s fetch path so the dropdown and the job list can never
/// disagree on what a selection offers.
pub fn sources_for_selection(
    ctx: &Context,
    sel: &Select<'_>,
    fetch: &mut dyn FnMut(&str) -> Result<String, Error>,
) -> Result<Vec<HostSource>, Error> {
    if sel.game.trim().is_empty() {
        return Err(Error::Usage("game slug required".to_string()));
    }
    let slug = crate::core::game_arg_slug(sel.game);
    let url = format!("https://lewdzone.com/game/{slug}/");
    let html = fetch(&url)?;
    let parsed = scraper::game::parse_game(&html);
    let settings = crate::core::settings::Settings::load(&ctx.config_path)?;
    sources_for(
        &parsed,
        sel.version,
        sel.platform,
        sel.tab,
        settings.source_priority.as_deref(),
    )
}

/// Resolve the go-link for one entry and build a canonical folder basename.
pub fn job_for(
    game: &Game,
    entry: &DownloadEntry,
    root: &std::path::Path,
    lzapps_root: &std::path::Path,
    resolve: &mut dyn FnMut(&str) -> Result<resolver::ResolvedUrl, Error>,
) -> Result<Job, Error> {
    crate::resolver::validate_host(&entry.host)?;
    let resolved = resolve(&entry.go_link)?;
    let variant = entry.variant.as_deref();
    let orig: &str = resolved
        .url
        .rsplit('/')
        .next()
        .unwrap_or_else(|| &resolved.url);
    let platform = entry.platform.as_deref().unwrap_or("pc").to_string();
    let base = folder::safe_basename(&game.title, &entry_label(entry), &platform, variant);
    let ext = folder::file_ext(orig);
    let target_name = format!("{base}.{ext}");
    let final_path = folder::engine_download_target(root, game.engine.as_deref(), &target_name);
    let install_dir = folder::engine_install_dir(lzapps_root, game.engine.as_deref(), &game.slug);
    Ok(Job {
        game: game.slug.clone(),
        title: game.title.clone(),
        post_id: game.post_id,
        engine: game.engine.clone(),
        version: entry_label(entry),
        platform,
        tab: entry.host.clone(),
        url: resolved.url,
        target: Some(final_path),
        install_dir: Some(install_dir),
    })
}

/// Entry label: drop hosting-host prefix noise down to the row label.
fn entry_label(e: &DownloadEntry) -> String {
    if e.label.trim().is_empty() {
        e.host.clone()
    } else {
        e.label.trim().to_string()
    }
}

/// The full CLI entry point (produces the exit code for the caller).
#[allow(clippy::too_many_arguments)]
pub fn run(
    ctx: &Context,
    game: &str,
    version: &str,
    platform: &str,
    tab: &str,
    source: Option<&str>,
    _resume: bool,
    _queue: bool,
) -> Result<crate::cli::ExitCode, Error> {
    let sel = Select {
        game,
        version,
        platform,
        tab,
        source,
    };
    let grace = crate::core::scheduler::grace_for(ctx);
    let mut last_pct = -1_i32;
    run_with(
        ctx,
        &sel,
        &mut |url| scraper::fetch(url),
        &mut resolver::resolve,
        &mut |job| {
            dispatch_with(job, &mut scraper::download_stream, &mut |done, total| {
                let pct = done.saturating_mul(100).checked_div(total).unwrap_or(0) as i32;
                if pct != last_pct {
                    last_pct = pct;
                    eprintln!("[download] {pct}%");
                }
                Ok(())
            })
        },
        grace,
        &mut |d| std::thread::sleep(d),
    )
}

/// Hosts whose revealed URL is a direct file (streamed inside the app with
/// byte progress). Everything else hands its resolved URL to the OS default
/// handler — the installed cloud app or the browser — with zero configuration.
/// Hosts whose revealed URL is a direct file (streamed inside the app with
/// byte progress). Everything else hands its resolved URL to the OS default
/// handler — the installed cloud app or the browser — with zero configuration.
pub const DIRECT_STREAM_HOSTS: &[&str] = &["fileknot", "pixeldrain", "mediafire", "workupload"];

/// Stream seam: takes a URL, returns `(total bytes, body reader)` (Rule 11).
pub type StreamFn<'a> = &'a mut dyn FnMut(&str) -> Result<(u64, Box<dyn Read>), Error>;

/// Progress callback: `(bytes done, bytes total)`.
pub type ProgressCallback<'a> = &'a mut dyn FnMut(u64, u64) -> Result<(), Error>;

/// True when `host` (a `Job::tab` slug) is a direct-file host we stream in-app.
pub fn is_direct_stream_host(host: &str) -> bool {
    DIRECT_STREAM_HOSTS
        .iter()
        .any(|h| h.eq_ignore_ascii_case(host))
}

/// Dispatch a single job: direct-file hosts stream into `job.target` (accelerated
/// with 4 concurrent chunk connections), everything else opens in the OS default handler.
/// Host allowlisting already happened at resolve time via `resolver::validate_host`, so we only
/// ever act on validated hosts (Rule 10). Shared by the synchronous CLI path and the async GUI queue worker.
pub fn dispatch(job: &Job) -> Result<(), Error> {
    if is_direct_stream_host(&job.tab) {
        return stream_target_accelerated(job, &mut |_, _| Ok(()));
    }
    crate::core::native::open_url(&job.url)
}

/// `dispatch` with the stream seam injected (Rule 11 offline fixtures).
pub fn dispatch_with(
    job: &Job,
    download: StreamFn<'_>,
    progress: ProgressCallback<'_>,
) -> Result<(), Error> {
    if is_direct_stream_host(&job.tab) {
        return stream_target(job, download, progress);
    }
    crate::core::native::open_url(&job.url)
}

/// Stream `download(url)` into `job.target`, reporting `(bytes done, total)`.
/// Fails without touching the network when the job has no computed target.
/// After a `.zip` finishes, it is extracted into `job.install_dir` and an
/// `app.json` manifest is written.
pub fn stream_target(
    job: &Job,
    download: StreamFn<'_>,
    progress: ProgressCallback<'_>,
) -> Result<(), Error> {
    let target = job
        .target
        .as_ref()
        .ok_or_else(|| Error::Usage("direct-stream job has no target path".to_string()))?;
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let (total, mut reader) = download(&job.url)?;
    let file = std::fs::File::create(target)?;
    let mut writer = std::io::BufWriter::with_capacity(1024 * 1024, file);
    let mut buf = vec![0_u8; 512 * 1024];
    let mut done: u64 = 0;
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        writer.write_all(&buf[..n])?;
        done += n as u64;
        progress(done, total)?;
    }
    writer.flush()?;

    if crate::core::library::is_supported_archive(target) {
        if let Some(install_dir) = &job.install_dir {
            let meta = extract::InstallMeta {
                slug: &job.game,
                post_id: job.post_id,
                title: &job.title,
                version: &job.version,
                platform: &job.platform,
                tab: &job.tab,
                engine: job.engine.as_deref(),
                download_url: &job.url,
            };
            extract::install_from_archive_with_progress(target, install_dir, &meta, progress)?;
        }
    }

    Ok(())
}

/// Accelerated download: streams `job.url` using 4 concurrent chunk connections
/// writing directly into `job.target` via OS positioned writes. When a supported
/// archive finishes, extracts into `job.install_dir` and writes an `app.json` manifest.
pub fn stream_target_accelerated(job: &Job, progress: ProgressCallback<'_>) -> Result<(), Error> {
    let target = job
        .target
        .as_ref()
        .ok_or_else(|| Error::Usage("direct-stream job has no target path".to_string()))?;
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }

    crate::scraper::download_file(&job.url, target, progress)?;

    if crate::core::library::is_supported_archive(target) {
        if let Some(install_dir) = &job.install_dir {
            let meta = extract::InstallMeta {
                slug: &job.game,
                post_id: job.post_id,
                title: &job.title,
                version: &job.version,
                platform: &job.platform,
                tab: &job.tab,
                engine: job.engine.as_deref(),
                download_url: &job.url,
            };
            extract::install_from_archive_with_progress(target, install_dir, &meta, progress)?;
        }
    }

    Ok(())
}

/// Everything except the network+spawn seams; wired for tests with fixtures.
///
/// Dispatching uses `core::scheduler::paced` so the batch starts exactly one
/// job at a time, waiting `grace` between starts (cloud free-tier throttle
/// protection). `sleep` is injected so tests assert pacing without waiting.
#[allow(clippy::too_many_arguments)]
pub fn run_with(
    ctx: &Context,
    sel: &Select<'_>,
    fetch: &mut dyn FnMut(&str) -> Result<String, Error>,
    resolve: &mut dyn FnMut(&str) -> Result<resolver::ResolvedUrl, Error>,
    dispatch: &mut dyn FnMut(&Job) -> Result<(), Error>,
    grace: std::time::Duration,
    sleep: &mut dyn FnMut(std::time::Duration),
) -> Result<crate::cli::ExitCode, Error> {
    let jobs = jobs_for(ctx, sel, fetch, resolve)?;
    crate::core::scheduler::paced(grace, &jobs, dispatch, sleep)?;

    print!("{}", serde_json::to_string_pretty(&jobs).unwrap());
    Ok(crate::cli::ExitCode::Ok)
}

/// Build the resolved job list for a selection WITHOUT dispatching or printing.
/// Shared by the CLI (which then dispatches + prints) and the Store GUI command
/// (which dispatches and returns the jobs to the view). Validates the game
/// slug and the download root before any job is built (dispatch-builder).
pub fn jobs_for(
    ctx: &Context,
    sel: &Select<'_>,
    fetch: &mut dyn FnMut(&str) -> Result<String, Error>,
    resolve: &mut dyn FnMut(&str) -> Result<resolver::ResolvedUrl, Error>,
) -> Result<Vec<Job>, Error> {
    if sel.game.trim().is_empty() {
        return Err(Error::Usage("game slug required".to_string()));
    }
    if crate::core::models::normalize_platform(sel.platform) == "android" {
        return Err(Error::Usage(
            "Android game downloads are not supported by the desktop app".to_string(),
        ));
    }
    // Canonical game URL (slug may itself be a full URL; normalize).
    let slug = crate::core::game_arg_slug(sel.game);
    let url = format!("https://lewdzone.com/game/{slug}/");
    let html = fetch(&url)?;
    let parsed = scraper::game::parse_game(&html);

    let settings = crate::core::settings::Settings::load(&ctx.config_path)?;
    let entries = pick_entries_with(
        &parsed,
        sel.version,
        sel.platform,
        sel.tab,
        settings.source_priority.as_deref(),
        sel.source,
    )?;
    let root = folder::download_root(ctx)?;
    let lzapps_root = folder::installed_root(ctx).or_else(|_| folder::lzapps_root(ctx))?;

    let mut jobs: Vec<Job> = Vec::new();
    for entry in entries {
        let job = job_for(&parsed, entry, &root, &lzapps_root, resolve)?;
        jobs.push(job);
    }
    Ok(jobs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::*;

    const GAME_FIXTURE: &str = include_str!("../../tests/fixtures/html/lz_game.html");

    fn sample_game() -> Game {
        scraper::game::parse_game(GAME_FIXTURE)
    }

    fn sample_entry(game: &Game) -> &DownloadEntry {
        game.versions
            .first()
            .and_then(|v| v.official.first())
            .expect("fixture game has an official entry")
    }

    fn stub_resolve(url: &str) -> Result<resolver::ResolvedUrl, Error> {
        Ok(resolver::ResolvedUrl {
            url: format!("https://fileknot.io/dl/{}", url),
            host: "fileknot".to_string(),
            platform: Some("pc".to_string()),
            version: None,
            game_id: None,
        })
    }

    #[test]
    fn pick_entries_finds_latest_official_windows() {
        let game = sample_game();
        let entries = pick_entries(&game, "latest", "PC", "official").unwrap();
        assert!(!entries.is_empty());
        assert_eq!(entries[0].host, "fileknot");
    }

    #[test]
    fn pick_entries_rejects_unknown_version() {
        let game = sample_game();
        let err = pick_entries(&game, "99.9", "PC", "official").unwrap_err();
        assert!(matches!(err, Error::Usage(_)));
        assert!(err.to_string().contains("1.0117"));
    }

    #[test]
    fn pick_entries_rejects_missing_platform() {
        let game = sample_game();
        let err = pick_entries(&game, "latest", "Atari", "official").unwrap_err();
        assert!(matches!(err, Error::Usage(_)));
        assert!(err.to_string().contains("atari"));
    }

    #[test]
    fn pick_entries_rejects_unknown_tab() {
        let game = sample_game();
        let err = pick_entries(&game, "latest", "PC", "weird").unwrap_err();
        assert!(matches!(err, Error::Usage(_)));
        assert!(err.to_string().contains("expected 'official'"));
    }

    #[test]
    fn pick_entries_orders_by_source_priority() {
        let game = sample_game();
        let plain = pick_entries(&game, "latest", "PC", "official").unwrap();
        assert!(plain.len() >= 2, "fixture has multiple official PC hosts");
        assert_eq!(plain[0].host, "fileknot");
        // Preferred host moves to the front, stable within equal ranks.
        let preferred = pick_entries_with(
            &game,
            "latest",
            "PC",
            "official",
            Some("transfaze,fileknot"),
            None,
        )
        .unwrap();
        assert_eq!(preferred[0].host, "transfaze");
        assert_eq!(preferred.last().unwrap().host, "fileknot");
        // Unpreferred beating a preferred slug: list mentions google only, which
        // is absent from official PC — site order must be preserved (knot→faze).
        let google_only =
            pick_entries_with(&game, "latest", "PC", "official", Some("google"), None).unwrap();
        let hosts: Vec<&str> = google_only.iter().map(|e| e.host.as_str()).collect();
        assert!(!hosts.is_empty());
        assert!(hosts.iter().all(|h| *h == "fileknot" || *h == "transfaze"));
        assert_eq!(&hosts[..2], &["fileknot", "transfaze"]);
    }

    #[test]
    fn pick_entries_restricts_to_selected_source() {
        let game = sample_game();
        let only =
            pick_entries_with(&game, "latest", "PC", "official", None, Some("fileknot")).unwrap();
        assert!(!only.is_empty(), "fixture has fileknot entries");
        assert!(
            only.iter().all(|e| e.host.eq_ignore_ascii_case("fileknot")),
            "every entry must be the selected source"
        );
        let missing = pick_entries_with(&game, "latest", "PC", "official", None, Some("mega"));
        assert!(matches!(missing, Err(Error::Usage(_))));
    }

    #[test]
    fn sources_for_lists_deduped_hosts_and_marks_preferred_default() {
        let game = sample_game();
        let sources = sources_for(&game, "latest", "PC", "official", None).unwrap();
        assert!(!sources.is_empty());
        assert_eq!(sources[0].host, "fileknot");
        assert!(!sources[0].preferred, "no priority = no preferred default");
        assert_eq!(sources[0].label, crate::core::native::app_name("fileknot"));
        // A configured preference that IS available becomes the default.
        let preferred = sources_for(
            &game,
            "latest",
            "PC",
            "official",
            Some("transfaze,fileknot"),
        )
        .unwrap();
        let first = preferred.first().unwrap();
        assert_eq!(first.host, "transfaze");
        assert!(
            preferred.iter().any(|s| s.preferred),
            "available preferred host must be marked"
        );
        let flagged = preferred.iter().find(|s| s.preferred).unwrap();
        assert_eq!(flagged.host, "transfaze");
        let count = preferred.iter().filter(|s| s.preferred).count();
        assert_eq!(count, 1, "exactly one preferred default");
    }

    #[test]
    fn job_for_builds_canonical_target() {
        let game = sample_game();
        let entry = sample_entry(&game);
        let root = std::path::Path::new("test-downloads-root");
        let lzapps = std::path::Path::new("test-lzapps-root");
        let job = job_for(&game, entry, root, lzapps, &mut stub_resolve).unwrap();
        assert!(job.url.starts_with("https://fileknot.io/"));
        let target = job.target.unwrap();
        assert!(target.starts_with(root));
        assert!(target.extension().is_some());
        let install = job.install_dir.unwrap();
        assert!(install.starts_with(lzapps));
        assert!(install.to_string_lossy().contains("treasure-of-nadia"));
    }

    #[test]
    fn jobs_for_rejects_android_platform() {
        let ctx = Context::new(
            std::env::temp_dir().join("lz-dl-android.db"),
            std::env::temp_dir().join("lz-dl-android-config.json"),
        );
        std::fs::write(&ctx.config_path, b"{}").unwrap();
        let sel = Select {
            game: "treasure-of-nadia",
            version: "latest",
            platform: "Android",
            tab: "official",
            source: None,
        };
        let err = jobs_for(
            &ctx,
            &sel,
            &mut |_| Ok(GAME_FIXTURE.to_string()),
            &mut stub_resolve,
        )
        .unwrap_err();
        assert!(matches!(err, Error::Usage(_)));
        assert!(err.to_string().contains("Android"));
    }

    fn write_zip(path: &std::path::Path, files: &[(&str, &[u8])]) {
        use std::io::Write;
        let file = std::fs::File::create(path).unwrap();
        let mut zip = zip::write::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        for (name, data) in files {
            zip.start_file(*name, options).unwrap();
            zip.write_all(data).unwrap();
        }
        zip.finish().unwrap();
    }

    #[test]
    fn stream_target_extracts_zip_to_install_dir() {
        let tmp = std::env::temp_dir().join(format!("lz-stream-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let source = tmp.join("source.zip");
        write_zip(&source, &[("game.exe", b"fake exe")]);
        let target = tmp.join("game.zip");
        let install = tmp.join("treasure-of-nadia");

        let job = Job {
            game: "treasure-of-nadia".into(),
            title: "Treasure of Nadia".into(),
            post_id: Some(1),
            engine: None,
            version: "1.0".into(),
            platform: "pc".into(),
            tab: "fileknot".into(),
            url: "https://fileknot.io/dl/abc".into(),
            target: Some(target.clone()),
            install_dir: Some(install.clone()),
        };

        let mut download = |_: &str| {
            let file = std::fs::File::open(&source)?;
            let total = file.metadata()?.len();
            Ok((total, Box::new(file) as Box<dyn Read>))
        };

        stream_target(&job, &mut download, &mut |_, _| Ok(())).unwrap();
        assert!(!target.exists(), "archive removed after extraction");
        assert!(install.join("game.exe").exists());
        assert!(install.join("app.json").exists());
    }

    #[test]
    fn run_with_offline_uses_fixture_fetch() {
        let ctx = Context::new(
            std::env::temp_dir().join("lz-dl.db"),
            std::env::temp_dir().join("lz-dl-config.json"),
        );
        let mut dispatched: Vec<String> = Vec::new();
        let sel = Select {
            game: "treasure-of-nadia",
            version: "latest",
            platform: "PC",
            tab: "official",
            source: None,
        };
        let code = run_with(
            &ctx,
            &sel,
            &mut |url| {
                assert!(url.ends_with("//lewdzone.com/game/treasure-of-nadia/"));
                Ok(GAME_FIXTURE.to_string())
            },
            &mut stub_resolve,
            &mut |job| {
                dispatched.push(job.url.clone());
                let _ =
                    std::fs::create_dir_all(job.target.as_ref().and_then(|p| p.parent()).unwrap());
                Ok(())
            },
            std::time::Duration::ZERO,
            &mut |_| {},
        );
        assert!(code.is_ok());
        assert_eq!(code.unwrap(), crate::cli::ExitCode::Ok);
        assert!(!dispatched.is_empty());
    }
}
