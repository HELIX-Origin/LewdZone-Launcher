//! Thin fetch helper (Rule 05) — the only network entry point in the scraper
//! family. Every parser stays pure `(html) -> model`; this module turns a
//! polite GET into the raw body every parser consumes.
//!
//! Etiquette baked in (Rule 05):
//! - one request per second per host (host-scoped clock)
//! - real browser-grade User-Agent
//! - 15s timeout per request
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
}
