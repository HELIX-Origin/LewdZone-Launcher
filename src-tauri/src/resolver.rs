//! Resolver family — turns a go-link (`https://lewdzone.com/go/#t=v1.…`)
//! into a real, actionable download URL through the site's two-step
//! `start` → `reveal` API (Rule 05).
//!
//! The `#fragment` token is never sent to any server; only `api.php` sees the
//! token, and only a resolved real URL is ever handed to a download manager
//! (Rule 07). Host validation keeps unknown hosts from reaching an adapter
//! (Rule 10.2).

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::core::Error;

/// Resolver API endpoint (`api.php`).
const ENDPOINT: &str = "https://lewdzone.com/go/api.php";
/// `Referer` the site's own go page would send (Rule 05).
const REFERER: &str = "https://lewdzone.com/go/";
/// Upper bound on reveal attempts gated by `retry_in` (mirrors site JS).
const MAX_REVEAL_ATTEMPTS: u32 = 4;
/// Bounded retries for transport-level failures (Rule 05: <= 3).
const MAX_TRANSPORT_RETRIES: u32 = 3;
/// Per-request timeout (Rule 05: 15s).
const TIMEOUT: Duration = Duration::from_secs(15);
/// Minimum gap between requests to the API host.
const HOST_INTERVAL: Duration = Duration::from_secs(1);

/// Allowlisted host slugs (mirrors the go.js `ICONS` list; re-verified against
/// the treasure-of-nadia fixture, Rule 10.2). Anything else is refused.
const KNOWN_HOSTS: &[&str] = &[
    "fileknot",
    "transfaze",
    "gofile",
    "mega",
    "mixdrop",
    "terminal",
    "uploadhaven",
    "workupload",
    "pixeldrain",
    "racaty",
    "mediafire",
];

/// Result of one successful resolution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedUrl {
    /// Real file URL, trailing literal `\r` stripped.
    pub url: String,
    /// Allowlisted host slug (e.g. `fileknot`).
    pub host: String,
    /// Platform label from the start response (e.g. `windows`).
    pub platform: Option<String>,
    /// Version label from the start response (e.g. `1.0117`).
    pub version: Option<String>,
    /// Game post id (e.g. `18212`).
    pub game_id: Option<i64>,
}

#[derive(Debug, Serialize)]
struct StartRequest<'a> {
    action: &'static str,
    token: &'a str,
}

#[derive(Debug, Deserialize)]
struct StartResponse {
    ok: bool,
    wait: Option<u32>,
    ticket: Option<String>,
    captcha: Option<u8>,
    host: Option<String>,
    platform: Option<String>,
    version: Option<String>,
    game: Option<i64>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct RevealRequest<'a> {
    action: &'static str,
    token: &'a str,
    ticket: &'a str,
}

#[derive(Debug, Deserialize)]
struct RevealResponse {
    ok: bool,
    url: Option<String>,
    retry_in: Option<u64>,
    captcha: Option<u8>,
    error: Option<String>,
}

/// Extract the `#t=` token from a go-link href. The fragment is the whole
/// tail of the URL (never sent to a server).
pub fn extract_token(go_link: &str) -> Option<&str> {
    go_link.split("#t=").nth(1).map(str::trim)
}

/// Allowlist gate (Rule 10.2): only hosts we have re-verified may reach a
/// download manager.
pub fn validate_host(host: &str) -> Result<(), Error> {
    if KNOWN_HOSTS.contains(&host) {
        Ok(())
    } else {
        Err(Error::Network(format!("unknown download host '{host}'")))
    }
}

fn host_of(url: &str) -> Option<&str> {
    url.split("://").nth(1)?.split('/').next()
}

/// Confirm a resolved URL is http(s) and its hostname belongs to the
/// allowlisted host slug from `start`.
fn verify_resolved(url: &str, host: &str) -> Result<(), Error> {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err(Error::Network(format!(
            "resolved URL has unexpected scheme: {url}"
        )));
    }
    let hostname = host_of(url).ok_or_else(|| Error::Network("resolved URL has no host".into()))?;
    if !hostname
        .to_ascii_lowercase()
        .contains(&host.to_ascii_lowercase())
    {
        return Err(Error::Network(format!(
            "resolved host '{hostname}' does not match token host '{host}'"
        )));
    }
    Ok(())
}

/// Resolve a go-link end to end. `post` is injected so unit tests run
/// offline with canned responses (Rule 11); the public `resolve` uses the
/// real `post_json`.
fn resolve_with(
    go_link: &str,
    post: &mut dyn FnMut(&str, &str) -> Result<String, Error>,
) -> Result<ResolvedUrl, Error> {
    let token =
        extract_token(go_link).ok_or_else(|| Error::Usage("go-link has no #t= token".into()))?;

    let start_body = serde_json::to_string(&StartRequest {
        action: "start",
        token,
    })
    .map_err(|e| Error::Network(format!("encode start: {e}")))?;
    let start: StartResponse = serde_json::from_str(&post(ENDPOINT, &start_body)?)
        .map_err(|e| Error::Network(format!("bad start response: {e}")))?;
    if start.captcha == Some(1) {
        return Err(Error::Network(
            "a captcha is required on this link and cannot be auto-solved".into(),
        ));
    }
    if !start.ok {
        return Err(Error::Network(
            start.error.unwrap_or_else(|| "start failed".into()),
        ));
    }
    let ticket = start
        .ticket
        .clone()
        .filter(|t| !t.is_empty())
        .ok_or_else(|| Error::Network("start response carried no ticket".into()))?;
    let host = start
        .host
        .clone()
        .ok_or_else(|| Error::Network("start response carried no host".into()))?;
    validate_host(&host)?;

    // Server-enforced gap before reveal (mirrors site JS: sleep(wait + 1)).
    let wait = u64::from(start.wait.unwrap_or(0)) + 1;
    std::thread::sleep(Duration::from_secs(wait));

    let mut attempt = 0u32;
    let url = loop {
        let reveal_body = serde_json::to_string(&RevealRequest {
            action: "reveal",
            token,
            ticket: &ticket,
        })
        .map_err(|e| Error::Network(format!("encode reveal: {e}")))?;
        let reveal: RevealResponse = serde_json::from_str(&post(ENDPOINT, &reveal_body)?)
            .map_err(|e| Error::Network(format!("bad reveal response: {e}")))?;
        if reveal.captcha == Some(1) {
            return Err(Error::Network(
                "a captcha is required on this link and cannot be auto-solved".into(),
            ));
        }
        if !reveal.ok {
            return Err(Error::Network(
                reveal.error.unwrap_or_else(|| "reveal failed".into()),
            ));
        }
        match reveal.url {
            Some(raw) => break raw,
            None => {
                if let Some(retry_in) = reveal.retry_in {
                    attempt += 1;
                    if attempt >= MAX_REVEAL_ATTEMPTS {
                        return Err(Error::Network("reveal retries exhausted".into()));
                    }
                    std::thread::sleep(Duration::from_secs(retry_in));
                    continue;
                }
                return Err(Error::Network("reveal returned no url".into()));
            }
        }
    };

    // The final URL arrives with a literal trailing `\r` — strip it.
    let url = url.trim_end_matches('\r').to_string();
    verify_resolved(&url, &host)?;

    Ok(ResolvedUrl {
        url,
        host,
        platform: start.platform,
        version: start.version,
        game_id: start.game,
    })
}

/// Resolve a go-link against the live API (polite, Rule 05).
pub fn resolve(go_link: &str) -> Result<ResolvedUrl, Error> {
    resolve_with(go_link, &mut post_json)
}

/// POST a JSON body to the API with full Rule 05 etiquette: real UA, the
/// site's Referer, 15s timeout, 1 req/s host clock, bounded retries.
fn post_json(endpoint: &str, body: &str) -> Result<String, Error> {
    static HOST_CLOCK: std::sync::OnceLock<std::sync::Mutex<Vec<(String, std::time::Instant)>>> =
        std::sync::OnceLock::new();
    static AGENT: std::sync::OnceLock<ureq::Agent> = std::sync::OnceLock::new();

    let clock = HOST_CLOCK.get_or_init(|| std::sync::Mutex::new(Vec::new()));
    {
        let mut clock = clock.lock().unwrap_or_else(|p| p.into_inner());
        let now = std::time::Instant::now();
        let mut wait = Duration::ZERO;
        for (h, at) in clock.iter() {
            if h == "lewdzone.com" && now.saturating_duration_since(*at) < HOST_INTERVAL {
                wait = wait.max(HOST_INTERVAL - now.saturating_duration_since(*at));
            }
        }
        if !wait.is_zero() {
            std::thread::sleep(wait);
        }
        clock.retain(|(h, at)| {
            h != "lewdzone.com" || now.saturating_duration_since(*at) < HOST_INTERVAL * 4
        });
        clock.push(("lewdzone.com".to_string(), now));
    }

    let agent = AGENT.get_or_init(|| {
        ureq::Agent::config_builder()
            .user_agent(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 LewdZoneLauncher/0.1.0",
            )
            .timeout_per_call(Some(TIMEOUT))
            .timeout_connect(Some(TIMEOUT))
            .timeout_recv_body(Some(TIMEOUT))
            .build()
            .new_agent()
    });

    let mut attempt = 0u32;
    loop {
        let result = agent
            .post(endpoint)
            .header("Content-Type", "application/json")
            .header("Referer", REFERER)
            .send(body);
        match result {
            Ok(resp) if resp.status() == 200 => {
                return resp
                    .into_body()
                    .read_to_string()
                    .map_err(|e| Error::Network(format!("{endpoint}: {e}")));
            }
            Ok(resp) => {
                attempt += 1;
                if attempt >= MAX_TRANSPORT_RETRIES {
                    return Err(Error::Network(format!("HTTP {}", resp.status())));
                }
            }
            Err(e) => {
                attempt += 1;
                if attempt >= MAX_TRANSPORT_RETRIES {
                    return Err(Error::Network(format!("{endpoint}: {e}")));
                }
            }
        }
        std::thread::sleep(Duration::from_millis(500 * u64::from(attempt)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOKEN_START: &str = r#"{"ok":true,"wait":3,"ticket":"eyJ0Ijoi","captcha":0,"host":"fileknot","platform":"windows","version":"1.0117","game":18212}"#;
    const TOKEN_REVEAL: &str =
        r#"{"ok":true,"url":"https://fileknot.io/0ddc033076217380/file.zip\r"}"#;

    fn canned(overrides: Vec<(String, serde_json::Value)>) -> String {
        let mut map = serde_json::Map::new();
        map.insert("ok".into(), true.into());
        map.insert("wait".into(), 3.into());
        map.insert("ticket".into(), "eyJ0Ijoi".into());
        map.insert("captcha".into(), 0.into());
        map.insert("host".into(), "fileknot".into());
        map.insert("platform".into(), "windows".into());
        map.insert("version".into(), "1.0117".into());
        map.insert("game".into(), 18212.into());
        for (k, v) in overrides {
            map.insert(k, v);
        }
        serde_json::to_string(&map).unwrap()
    }

    #[test]
    fn extracts_token_after_hash_t() {
        assert_eq!(
            extract_token("https://lewdzone.com/go/#t=v1.abc.xyz"),
            Some("v1.abc.xyz")
        );
        assert!(extract_token("https://lewdzone.com/go/").is_none());
    }

    #[test]
    fn allowlist_rejects_unknown_host() {
        assert!(validate_host("fileknot").is_ok());
        assert!(validate_host("mega").is_ok());
        let err = validate_host("evil.example").unwrap_err();
        assert!(matches!(err, Error::Network(_)));
    }

    #[test]
    fn resolves_ok_path_and_strips_trailing_cr() {
        let mut post = |_endpoint: &str, body: &str| -> Result<String, Error> {
            if body.contains("reveal") {
                Ok(TOKEN_REVEAL.to_string())
            } else {
                Ok(TOKEN_START.to_string())
            }
        };
        let r = resolve_with("https://lewdzone.com/go/#t=v1.x.y", &mut post).unwrap();
        assert_eq!(r.url, "https://fileknot.io/0ddc033076217380/file.zip");
        assert!(!r.url.ends_with('\r'));
        assert_eq!(r.host, "fileknot");
        assert_eq!(r.platform.as_deref(), Some("windows"));
        assert_eq!(r.version.as_deref(), Some("1.0117"));
        assert_eq!(r.game_id, Some(18212));
    }

    #[test]
    fn retry_in_gates_reveal_and_then_succeeds() {
        let mut count = 0;
        let mut post = |_endpoint: &str, body: &str| -> Result<String, Error> {
            if body.contains("reveal") {
                count += 1;
                if count == 1 {
                    Ok(r#"{"ok":true,"retry_in":1}"#.to_string())
                } else {
                    Ok(TOKEN_REVEAL.to_string())
                }
            } else {
                Ok(TOKEN_START.to_string())
            }
        };
        let r = resolve_with("https://lewdzone.com/go/#t=v1.x.y", &mut post).unwrap();
        assert_eq!(r.host, "fileknot");
        assert_eq!(count, 2);
    }

    #[test]
    fn retry_exhaustion_errors() {
        let mut post = |_endpoint: &str, body: &str| -> Result<String, Error> {
            if body.contains("reveal") {
                Ok(r#"{"ok":true,"retry_in":1}"#.to_string())
            } else {
                Ok(TOKEN_START.to_string())
            }
        };
        let err = resolve_with("https://lewdzone.com/go/#t=v1.x.y", &mut post).unwrap_err();
        assert!(matches!(err, Error::Network(_)));
        assert!(err.to_string().contains("exhausted"));
    }

    #[test]
    fn bad_token_start_errors() {
        let mut post = |_endpoint: &str, _body: &str| -> Result<String, Error> {
            Ok(r#"{"ok":false,"error":"bad token"}"#.to_string())
        };
        let err = resolve_with("https://lewdzone.com/go/#t=v1.x.y", &mut post).unwrap_err();
        assert!(matches!(err, Error::Network(_)));
        assert!(err.to_string().contains("bad token"));
    }

    #[test]
    fn captcha_raises_user_visible_error() {
        let mut post = |_endpoint: &str, _body: &str| -> Result<String, Error> {
            Ok(canned(vec![("captcha".into(), 1.into())]))
        };
        let err = resolve_with("https://lewdzone.com/go/#t=v1.x.y", &mut post).unwrap_err();
        assert!(err.to_string().contains("captcha"));
    }

    #[test]
    fn no_token_is_usage_error() {
        let mut post =
            |_endpoint: &str, _body: &str| -> Result<String, Error> { Ok("".to_string()) };
        let err = resolve_with("https://lewdzone.com/go/", &mut post).unwrap_err();
        assert!(matches!(err, Error::Usage(_)));
    }

    #[test]
    fn missing_ticket_errors() {
        let mut post = |_endpoint: &str, _body: &str| -> Result<String, Error> {
            Ok(canned(vec![("ticket".into(), serde_json::Value::Null)]))
        };
        let err = resolve_with("https://lewdzone.com/go/#t=v1.x.y", &mut post).unwrap_err();
        assert!(matches!(err, Error::Network(_)));
    }

    #[test]
    fn unknown_host_is_refused() {
        let mut post = |_endpoint: &str, _body: &str| -> Result<String, Error> {
            Ok(canned(vec![("host".into(), "sketchy".into())]))
        };
        let err = resolve_with("https://lewdzone.com/go/#t=v1.x.y", &mut post).unwrap_err();
        assert!(matches!(err, Error::Network(_)));
        assert!(err.to_string().contains("unknown download host"));
    }

    #[test]
    fn mismatched_resolved_host_fails_verification() {
        let mut post = |_endpoint: &str, body: &str| -> Result<String, Error> {
            if body.contains("reveal") {
                Ok(r#"{"ok":true,"url":"https://evil.example/file.zip\r"}"#.to_string())
            } else {
                Ok(TOKEN_START.to_string())
            }
        };
        let err = resolve_with("https://lewdzone.com/go/#t=v1.x.y", &mut post).unwrap_err();
        assert!(matches!(err, Error::Network(_)));
        assert!(err.to_string().contains("does not match"));
    }
}
