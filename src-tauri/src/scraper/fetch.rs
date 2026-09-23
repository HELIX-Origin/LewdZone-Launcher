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
        let total = resp.body().content_length().unwrap_or(0);
        let reader: Box<dyn std::io::Read> = Box::new(resp.into_body().into_reader());
        return Ok((total, reader));
    }
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
}
