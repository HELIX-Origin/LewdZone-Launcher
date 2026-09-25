//! Thin fetch helper (Rule 05) — the only network entry point in the scraper
//! family. Every parser stays pure `(html) -> model`; this module turns a
//! polite GET into the raw body every parser consumes, and streams binary
//! downloads for the in-app direct-file path (`download::DIRECT_STREAM_HOSTS`).
//!
//! Etiquette baked in (Rule 05):
//! - one request per second per host (host-scoped clock)
//! - real browser-grade User-Agent
//! - 15s timeout per request / response headers
//! - bounded retries (<=3) with linear backoff on transient failures

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::core::Error;

/// Browser-grade UA so lewdzone.com serves the normal layout.
const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 LewdZoneLauncher/0.1.0";
/// Per-request timeout (Rule 05: 15s).
const TIMEOUT: Duration = Duration::from_secs(15);
/// Bounded retries (Rule 05: <= 3).
const MAX_RETRIES: u32 = 3;
/// Minimum gap between two requests to the same host.
const HOST_INTERVAL: Duration = Duration::from_secs(1);

/// Per-host last-request timestamps (host-scoped 1 req/s clock).
static HOST_CLOCK: OnceLock<Mutex<Vec<(String, Instant)>>> = OnceLock::new();

fn host_clock() -> &'static Mutex<Vec<(String, Instant)>> {
    HOST_CLOCK.get_or_init(|| Mutex::new(Vec::new()))
}

/// Shared agent built once with Rule 05 timeouts + real UA.
static AGENT: OnceLock<ureq::Agent> = OnceLock::new();

fn agent() -> &'static ureq::Agent {
    AGENT.get_or_init(|| {
        ureq::Agent::config_builder()
            .user_agent(USER_AGENT)
            .timeout_per_call(Some(TIMEOUT))
            .timeout_connect(Some(TIMEOUT))
            .timeout_recv_body(Some(TIMEOUT))
            .build()
            .new_agent()
    })
}

fn host_of(url: &str) -> &str {
    match url.split("://").nth(1) {
        Some(rest) => rest.split('/').next().unwrap_or(""),
        None => url.split('/').next().unwrap_or(""),
    }
}

/// Enforce the host-scoped rate limit: sleep until >=1s since our last request
/// to `host`, then record this request as the new baseline.
fn enforce_rate(host: &str) {
    let mut clock = host_clock().lock().unwrap_or_else(|p| p.into_inner());
    let now = Instant::now();
    let mut wait = Duration::ZERO;
    for (h, at) in clock.iter() {
        if h == host && now.saturating_duration_since(*at) < HOST_INTERVAL {
            wait = wait.max(HOST_INTERVAL - now.saturating_duration_since(*at));
        }
    }
    if !wait.is_zero() {
        std::thread::sleep(wait);
    }
    clock.retain(|(h, at)| h != host || now.saturating_duration_since(*at) < HOST_INTERVAL * 4);
    clock.push((host.to_string(), Instant::now()));
}

/// Fetch a URL body with full Rule 05 etiquette. Returns UTF-8 text for HTML
/// pages; binary payloads are handled by `fetch_bytes`.
pub fn fetch(url: &str) -> Result<String, Error> {
    let host = host_of(url).to_string();
    enforce_rate(&host);
    let mut attempt = 0u32;
    loop {
        match try_fetch_text(url) {
            Ok(body) => return Ok(body),
            Err(err) => {
                attempt += 1;
                if attempt >= MAX_RETRIES {
                    return Err(Error::Network(format!("{url}: {err}")));
                }
                std::thread::sleep(Duration::from_millis(500 * u64::from(attempt)));
            }
        }
    }
}

fn try_fetch_text(url: &str) -> Result<String, String> {
    let resp = agent().get(url).call().map_err(|e| e.to_string())?;
    if resp.status() != 200 {
        return Err(format!("HTTP {}", resp.status()));
    }
    let body: String = resp
        .into_body()
        .read_to_string()
        .map_err(|e| e.to_string())?;
    Ok(body)
}

/// Bounded hops for manually followed redirects (each one host-validated).
const MAX_REDIRECT_HOPS: u32 = 3;

/// Shared agent for binary downloads: no total-call/body budgets (game files
/// are large — `timeout_recv_body`/`timeout_per_call` are checked on every
/// body read and would abort mid-file), and **zero automatic redirects** so
/// every hop is followed manually and host-validated (Rule 10.2).
static DOWNLOAD_AGENT: OnceLock<ureq::Agent> = OnceLock::new();

fn download_agent() -> &'static ureq::Agent {
    DOWNLOAD_AGENT.get_or_init(|| {
        ureq::Agent::config_builder()
            .user_agent(USER_AGENT)
            .timeout_connect(Some(TIMEOUT))
            .timeout_recv_response(Some(TIMEOUT))
            .timeout_recv_body(None)
            .max_redirects(0)
            .build()
            .new_agent()
    })
}

/// Split `https://host/path` into `("https", "host")`; `None` when `url` is
/// not an absolute http(s) URL.
fn origin_of(url: &str) -> Option<(&str, &str)> {
    let (scheme, rest) = url.split_once("://")?;
    let host = rest.split('/').next().unwrap_or("");
    if scheme.is_empty() || host.is_empty() {
        return None;
    }
    Some((scheme, host))
}

/// Resolve a redirect `location` against `current`, refusing any hop that
/// leaves the original scheme/host (subdomains of the original host are OK).
/// This is the interception guard: an ad domain spliced into a redirect can
/// never receive the request (user decision 2026-09, Rule 10.2).
fn redirect_target(current: &str, location: &str, base: (&str, &str)) -> Result<String, Error> {
    let (base_scheme, base_host) = base;
    let next = if location.starts_with("https://") || location.starts_with("http://") {
        location.to_string()
    } else if location.starts_with("//") {
        format!("{base_scheme}:{location}")
    } else if location.starts_with('/') {
        let authority = origin_of(current).map(|(_, h)| h).unwrap_or(base_host);
        format!("{base_scheme}://{authority}{location}")
    } else if location.contains("://") {
        return Err(Error::Network(format!(
            "redirect to '{location}' refused — only http(s) same-host redirects are followed"
        )));
    } else {
        return Err(Error::Network(format!(
            "{current}: unresolvable redirect location '{location}'"
        )));
    };
    let Some((scheme, host)) = origin_of(&next) else {
        return Err(Error::Network(format!(
            "{current}: redirect location '{location}' is not an absolute URL"
        )));
    };
    let host_l = host.to_ascii_lowercase();
    let base_l = base_host.to_ascii_lowercase();
    let same_site = host_l == base_l
        || (host_l.ends_with(&base_l)
            && host_l
                .as_bytes()
                .get(host_l.len() - base_l.len() - 1)
                .is_some_and(|b| *b == b'.'));
    if scheme != base_scheme || !same_site {
        return Err(Error::Network(format!(
            "redirect to '{scheme}://{host}' refused — downloads must stay on '{base_host}'"
        )));
    }
    Ok(next)
}

/// Stream a binary download with Rule 05 etiquette. Automatic redirects are
/// disabled; each hop is followed manually and must stay on the original
/// scheme + host (or a subdomain), so an intercepted redirect can never reach
/// an ad domain. Returns `(content_length, reader)` — the length is `0` when
/// the server omits it (progress then reports bytes only). Transport failures
/// retry <=3 times; once a response arrives, statuses and hops are handled
/// without retry (Rule 11: tests never hit this path — they inject seams).
pub fn download_stream(url: &str) -> Result<(u64, Box<dyn std::io::Read>), Error> {
    let Some(base) = origin_of(url) else {
        return Err(Error::Network(format!(
            "{url}: not an absolute http(s) URL"
        )));
    };
    let mut current = url.to_string();
    let mut hops = 0;
    loop {
        let host = host_of(&current).to_string();
        enforce_rate(&host);
        let resp = {
            let mut attempt = 0u32;
            loop {
                match download_agent().get(&current).call() {
                    Ok(r) => break r,
                    Err(err) => {
                        attempt += 1;
                        if attempt >= MAX_RETRIES {
                            return Err(Error::Network(format!("{current}: {err}")));
                        }
                        std::thread::sleep(Duration::from_millis(500 * u64::from(attempt)));
                    }
                }
            }
        };
        let code = resp.status().as_u16();
        if (300..400).contains(&code) {
            hops += 1;
            if hops > MAX_REDIRECT_HOPS {
                return Err(Error::Network(format!("{current}: too many redirects")));
            }
            let location = resp
                .headers()
                .get("location")
                .and_then(|v| v.to_str().ok())
                .filter(|s| !s.is_empty())
                .ok_or_else(|| {
                    Error::Network(format!("{current}: redirect without a location header"))
                })?
                .to_string();
            current = redirect_target(&current, &location, base)?;
            continue;
        }
        if !(200..300).contains(&code) {
            return Err(Error::Network(format!("{current}: HTTP {}", resp.status())));
        }
        if let Some(ct) = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
        {
            if ct.starts_with("text/html") {
                return Err(Error::Network(format!(
                    "{current}: server returned an HTML webpage instead of a binary file (the host requires interactive download)"
                )));
            }
        }
        let total = resp.body().content_length().unwrap_or(0);
        let reader: Box<dyn std::io::Read> = Box::new(resp.into_body().into_reader());
        return Ok((total, reader));
    }
}

/// Parse total size from an HTTP Content-Range header (e.g. `bytes 0-0/104857600`).
pub fn parse_content_range_total(header_val: &str) -> Option<u64> {
    let slash = header_val.rfind('/')?;
    let total_part = header_val[slash + 1..].trim();
    total_part.parse::<u64>().ok()
}

#[cfg(target_os = "windows")]
fn write_at(file: &std::fs::File, buf: &[u8], offset: u64) -> std::io::Result<()> {
    use std::os::windows::fs::FileExt;
    let mut written = 0;
    while written < buf.len() {
        let n = file.seek_write(&buf[written..], offset + written as u64)?;
        if n == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::WriteZero,
                "zero bytes written",
            ));
        }
        written += n;
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn write_at(file: &std::fs::File, buf: &[u8], offset: u64) -> std::io::Result<()> {
    use std::os::unix::fs::FileExt;
    file.write_all_at(buf, offset)
}

/// Accelerated file download: detects byte-range support (HTTP 206) and splits large files
/// (>= 4MB) into 4 concurrent chunk streams writing directly into the preallocated target file
/// via OS positioned writes. Falls back to an optimized single stream with 512KB I/O buffers
/// when ranges are not supported or files are small.
pub fn download_file(
    url: &str,
    target: &std::path::Path,
    progress: &mut dyn FnMut(u64, u64) -> Result<(), Error>,
) -> Result<u64, Error> {
    let Some(base) = origin_of(url) else {
        return Err(Error::Network(format!(
            "{url}: not an absolute http(s) URL"
        )));
    };
    let mut current = url.to_string();
    let mut hops = 0;
    loop {
        let host = host_of(&current).to_string();
        enforce_rate(&host);
        let resp = {
            let mut attempt = 0u32;
            loop {
                match download_agent()
                    .get(&current)
                    .header("Range", "bytes=0-0")
                    .call()
                {
                    Ok(r) => break r,
                    Err(err) => {
                        attempt += 1;
                        if attempt >= MAX_RETRIES {
                            return Err(Error::Network(format!("{current}: {err}")));
                        }
                        std::thread::sleep(Duration::from_millis(500 * u64::from(attempt)));
                    }
                }
            }
        };

        let code = resp.status().as_u16();
        if (300..400).contains(&code) {
            hops += 1;
            if hops > MAX_REDIRECT_HOPS {
                return Err(Error::Network(format!("{current}: too many redirects")));
            }
            let location = resp
                .headers()
                .get("location")
                .and_then(|v| v.to_str().ok())
                .filter(|s| !s.is_empty())
                .ok_or_else(|| {
                    Error::Network(format!("{current}: redirect without a location header"))
                })?
                .to_string();
            current = redirect_target(&current, &location, base)?;
            continue;
        }

        if !(200..300).contains(&code) {
            return Err(Error::Network(format!("{current}: HTTP {}", resp.status())));
        }

        if let Some(ct) = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
        {
            if ct.starts_with("text/html") {
                return Err(Error::Network(format!(
                    "{current}: server returned an HTML webpage instead of a binary file"
                )));
            }
        }

        // If server supported 206 Partial Content:
        if code == 206 {
            let content_range = resp
                .headers()
                .get("content-range")
                .and_then(|v| v.to_str().ok());
            let total_opt = content_range.and_then(parse_content_range_total);

            if let Some(total) = total_opt {
                if total >= 4 * 1024 * 1024 {
                    // Drop probe response so we free the connection before spawning workers
                    drop(resp);
                    return download_file_parallel(&current, target, total, progress);
                }
            }
        }

        // Single-stream fallback (e.g. 200 OK or file < 4MB)
        return download_file_single(resp, target, progress);
    }
}

fn download_file_single(
    resp: ureq::http::Response<ureq::Body>,
    target: &std::path::Path,
    progress: &mut dyn FnMut(u64, u64) -> Result<(), Error>,
) -> Result<u64, Error> {
    use std::io::{Read, Write};
    let total = resp.body().content_length().unwrap_or(0);
    let mut reader = resp.into_body().into_reader();
    let file = std::fs::File::create(target)?;
    let mut writer = std::io::BufWriter::with_capacity(1024 * 1024, file);
    let mut buf = vec![0_u8; 512 * 1024];
    let mut done = 0u64;
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
    Ok(done)
}

fn download_file_parallel(
    final_url: &str,
    target: &std::path::Path,
    total: u64,
    progress: &mut dyn FnMut(u64, u64) -> Result<(), Error>,
) -> Result<u64, Error> {
    use std::io::Read;
    use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
    use std::sync::Mutex;

    let file = std::fs::File::create(target)?;
    file.set_len(total)?;
    let file = std::sync::Arc::new(file);

    let num_chunks = 4;
    let chunk_size = total / num_chunks as u64;
    let mut ranges = Vec::new();
    for i in 0..num_chunks {
        let start = i as u64 * chunk_size;
        let end = if i == num_chunks - 1 {
            total - 1
        } else {
            (i as u64 + 1) * chunk_size - 1
        };
        ranges.push((start, end));
    }

    let total_done = AtomicU64::new(0);
    let cancel = AtomicBool::new(false);
    let first_error = Mutex::new(None::<String>);
    let active_workers = AtomicUsize::new(num_chunks);

    let cancel_ref = &cancel;
    let total_done_ref = &total_done;
    let first_error_ref = &first_error;
    let active_workers_ref = &active_workers;

    std::thread::scope(|s| {
        // Spawn parallel chunk workers
        for (start, end) in ranges {
            let file_ref = std::sync::Arc::clone(&file);
            s.spawn(move || {
                let mut offset = start;
                let mut buf = vec![0_u8; 512 * 1024];
                let mut attempt = 0u32;

                while offset <= end {
                    if cancel_ref.load(Ordering::Relaxed) {
                        active_workers_ref.fetch_sub(1, Ordering::SeqCst);
                        return;
                    }

                    let resp = download_agent()
                        .get(final_url)
                        .header("Range", &format!("bytes={offset}-{end}"))
                        .call();

                    if let Ok(r) = resp {
                        let mut reader = r.into_body().into_reader();
                        let mut read_failed = false;
                        while offset <= end {
                            if cancel_ref.load(Ordering::Relaxed) {
                                active_workers_ref.fetch_sub(1, Ordering::SeqCst);
                                return;
                            }
                            let to_read =
                                std::cmp::min(buf.len() as u64, end + 1 - offset) as usize;
                            if to_read == 0 {
                                break;
                            }
                            match reader.read(&mut buf[..to_read]) {
                                Ok(0) => {
                                    if offset <= end {
                                        read_failed = true;
                                    }
                                    break;
                                }
                                Ok(n) => {
                                    if let Err(e) = write_at(&file_ref, &buf[..n], offset) {
                                        let mut err_guard = first_error_ref.lock().unwrap();
                                        if err_guard.is_none() {
                                            *err_guard = Some(format!("write error: {e}"));
                                        }
                                        cancel_ref.store(true, Ordering::SeqCst);
                                        active_workers_ref.fetch_sub(1, Ordering::SeqCst);
                                        return;
                                    }
                                    offset += n as u64;
                                    total_done_ref.fetch_add(n as u64, Ordering::Relaxed);
                                }
                                Err(_) => {
                                    read_failed = true;
                                    break;
                                }
                            }
                        }

                        if !read_failed && offset > end {
                            // Chunk finished successfully
                            active_workers_ref.fetch_sub(1, Ordering::SeqCst);
                            return;
                        }
                    }

                    attempt += 1;
                    if attempt >= MAX_RETRIES {
                        let mut err_guard = first_error_ref.lock().unwrap();
                        if err_guard.is_none() {
                            *err_guard =
                                Some(format!("chunk at {offset} failed after {attempt} attempts"));
                        }
                        cancel_ref.store(true, Ordering::SeqCst);
                        active_workers_ref.fetch_sub(1, Ordering::SeqCst);
                        return;
                    }
                    std::thread::sleep(Duration::from_millis(500 * u64::from(attempt)));
                }
                active_workers_ref.fetch_sub(1, Ordering::SeqCst);
            });
        }

        // Main thread periodically calls progress callback
        while active_workers.load(Ordering::Relaxed) > 0 && !cancel.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_millis(150));
            let done = total_done.load(Ordering::Relaxed);
            if done >= total {
                break;
            }
            if let Err(e) = progress(done, total) {
                cancel.store(true, Ordering::SeqCst);
                let mut err_guard = first_error.lock().unwrap();
                if err_guard.is_none() {
                    *err_guard = Some(e.to_string());
                }
                break;
            }
        }
    });

    if let Some(err) = first_error.into_inner().unwrap() {
        let _ = std::fs::remove_file(target);
        if err.contains("cancelled") {
            return Err(Error::Usage(err));
        }
        return Err(Error::Network(err));
    }

    progress(total, total)?;
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_of_strips_scheme_and_path() {
        assert_eq!(
            host_of("https://lewdzone.com/games/page/2/?sort=Popularity"),
            "lewdzone.com"
        );
        assert_eq!(
            host_of("https://h1.lzcdn.com/img/cover.jpg"),
            "h1.lzcdn.com"
        );
    }

    #[test]
    fn fetch_errors_are_bounded_retries_and_typed() {
        // No network in unit tests (Rule 11): a dangling port fails fast and,
        // after MAX_RETRIES, surfaces as a typed Network error, not a panic.
        let err = fetch("http://127.0.0.1:1/").unwrap_err();
        assert!(matches!(err, Error::Network(_)));
    }

    #[test]
    fn origin_of_splits_scheme_and_host_only_when_absolute() {
        assert_eq!(
            origin_of("https://fileknot.io/dl/abc/file.zip"),
            Some(("https", "fileknot.io"))
        );
        assert_eq!(
            origin_of("http://h.example:8080/x"),
            Some(("http", "h.example:8080"))
        );
        assert_eq!(origin_of("/relative/path"), None);
        assert_eq!(origin_of("fileknot.io/x"), None);
        assert_eq!(origin_of("https://"), None);
    }

    #[test]
    fn redirect_target_allows_same_host_subdomain_and_root_relative() {
        let base = ("https", "fileknot.io");
        assert_eq!(
            redirect_target("https://fileknot.io/dl/a", "/dl/b", base).unwrap(),
            "https://fileknot.io/dl/b"
        );
        assert_eq!(
            redirect_target(
                "https://fileknot.io/dl/a",
                "https://cdn.fileknot.io/b",
                base
            )
            .unwrap(),
            "https://cdn.fileknot.io/b"
        );
        assert_eq!(
            redirect_target("https://fileknot.io/dl/a", "//s.fileknot.io/b", base).unwrap(),
            "https://s.fileknot.io/b"
        );
    }

    #[test]
    fn redirect_target_refuses_interception_to_other_domains() {
        let base = ("https", "fileknot.io");
        // Cross-host ad domain (the intercepted-redirect case, user 2026-09).
        let err = redirect_target(
            "https://fileknot.io/dl/a",
            "https://evil.example/payload.zip",
            base,
        )
        .unwrap_err();
        assert!(matches!(err, Error::Network(_)));
        assert!(err.to_string().contains("evil.example"));
        // Suffix trick: ends with the base host but not on a dot boundary.
        assert!(
            redirect_target("https://fileknot.io/dl/a", "https://notfileknot.io/x", base).is_err()
        );
        // Scheme downgrade and non-http schemes are refused.
        assert!(redirect_target("https://fileknot.io/dl/a", "http://fileknot.io/x", base).is_err());
        assert!(redirect_target("https://fileknot.io/dl/a", "ftp://fileknot.io/x", base).is_err());
        // Unresolvable relative junk and scheme-relative escape attempts.
        assert!(redirect_target("https://fileknot.io/dl/a", "payload.zip", base).is_err());
        assert!(redirect_target("https://fileknot.io/dl/a", "//evil.example/x", base).is_err());
    }

    #[test]
    fn test_parse_content_range_total() {
        assert_eq!(
            parse_content_range_total("bytes 0-0/104857600"),
            Some(104857600)
        );
        assert_eq!(
            parse_content_range_total("bytes 100-200/5242880"),
            Some(5242880)
        );
        assert_eq!(parse_content_range_total("bytes 0-0/*"), None);
        assert_eq!(parse_content_range_total("invalid"), None);
    }
}
