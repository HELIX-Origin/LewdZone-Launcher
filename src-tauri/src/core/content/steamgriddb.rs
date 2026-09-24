//! SteamGridDB provider: icons, grids, heroes, and logos.
//!
//! Requires `sgdb-api-key` secret in SQLite secret table. API docs:
//! https://www.steamgriddb.com/api/v2

use serde::Deserialize;
use std::collections::BTreeMap;
use std::io::Read;

use crate::core::models::GameCard;
use crate::core::Error;
use crate::scraper;

use super::{ArtworkKind, Enrichment, Provider};

pub struct SteamGridDb;

impl Provider for SteamGridDb {
    fn name(&self) -> &'static str {
        "steamgriddb"
    }

    fn enabled(&self, secrets: &BTreeMap<String, String>) -> bool {
        secrets
            .get("sgdb-api-key")
            .map(|k| !k.trim().is_empty())
            .unwrap_or(false)
    }

    fn enrich(
        &self,
        _secrets: &BTreeMap<String, String>,
        _card: &GameCard,
    ) -> Result<Option<Enrichment>, Error> {
        Ok(Some(Enrichment::default()))
    }

    fn artwork(
        &self,
        secrets: &BTreeMap<String, String>,
        card: &GameCard,
        kind: ArtworkKind,
    ) -> Result<Option<(String, Vec<u8>)>, Error> {
        let Some(key) = secrets.get("sgdb-api-key").filter(|k| !k.trim().is_empty()) else {
            return Ok(None);
        };

        let term = percent_encode(&card.title);
        let search_url = format!("https://www.steamgriddb.com/api/v2/search/autocomplete/{term}");
        let json = match authenticated_get(&search_url, key) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("[steamgriddb] search failed for '{}': {e}", card.title);
                return Ok(None);
            }
        };
        let resp: SearchResponse = match serde_json::from_str(&json) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[steamgriddb] search json parse error: {e}");
                return Ok(None);
            }
        };

        let Some(game) = resp.data.into_iter().next() else {
            return Ok(None);
        };

        let endpoint = match kind {
            ArtworkKind::Icon => "icons",
            ArtworkKind::Cover => "grids",
            ArtworkKind::Background => "heroes",
        };
        let url = format!(
            "https://www.steamgriddb.com/api/v2/{endpoint}/game/{id}?dimensions=512&nsfw=true",
            id = game.id
        );
        let json = match authenticated_get(&url, key) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("[steamgriddb] {endpoint} request failed: {e}");
                return Ok(None);
            }
        };
        let art: ArtResponse = match serde_json::from_str(&json) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("[steamgriddb] {endpoint} json parse error: {e}");
                return Ok(None);
            }
        };

        for item in art.data {
            if item.url.is_empty() {
                continue;
            }
            match scraper::download_stream(&item.url) {
                Ok((_, mut reader)) => {
                    let mut bytes = Vec::new();
                    if reader.read_to_end(&mut bytes).is_ok() && !bytes.is_empty() {
                        return Ok(Some((item.url, bytes)));
                    }
                }
                Err(e) => eprintln!("[steamgriddb] failed to download {}: {}", item.url, e),
            }
        }

        Ok(None)
    }
}

fn authenticated_get(url: &str, key: &str) -> Result<String, Error> {
    let agent = ureq::Agent::config_builder()
        .timeout_connect(Some(std::time::Duration::from_secs(10)))
        .timeout_recv_response(Some(std::time::Duration::from_secs(10)))
        .timeout_recv_body(Some(std::time::Duration::from_secs(10)))
        .build()
        .new_agent();

    let resp = agent
        .get(url)
        .header("Authorization", &format!("Bearer {key}"))
        .header("Accept", "application/json")
        .call()
        .map_err(|e| Error::Network(format!("{url}: {e}")))?;

    if resp.status() != 200 {
        return Err(Error::Network(format!("{url}: HTTP {}", resp.status())));
    }

    let body = resp
        .into_body()
        .read_to_string()
        .map_err(|e| Error::Network(format!("{url}: body read failed: {e}")))?;
    Ok(body)
}

fn percent_encode(s: &str) -> String {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    utf8_percent_encode(s, NON_ALPHANUMERIC).to_string()
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    #[serde(default)]
    data: Vec<SearchResult>,
}

#[derive(Debug, Deserialize)]
struct SearchResult {
    id: i64,
    #[allow(dead_code)]
    name: String,
}

#[derive(Debug, Deserialize)]
struct ArtResponse {
    #[serde(default)]
    data: Vec<ArtItem>,
}

#[derive(Debug, Deserialize)]
struct ArtItem {
    #[serde(default)]
    url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_name_and_enabled_requires_key() {
        let p = SteamGridDb;
        assert_eq!(p.name(), "steamgriddb");
        let mut secrets = BTreeMap::new();
        assert!(!p.enabled(&secrets));
        secrets.insert("sgdb-api-key".to_string(), "   ".to_string());
        assert!(!p.enabled(&secrets));
        secrets.insert("sgdb-api-key".to_string(), "valid_key".to_string());
        assert!(p.enabled(&secrets));
    }
}
