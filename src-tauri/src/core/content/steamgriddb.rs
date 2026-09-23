//! SteamGridDB provider: icons, grids, heroes, and logos.
//!
//! Requires `sgdb-api-key` secret. API docs:
//! https://www.steamgriddb.com/api/v2

use serde::Deserialize;

use crate::core::models::GameCard;
use crate::core::Error;
use crate::scraper;

use super::{ArtworkKind, Enrichment, Provider};

pub struct SteamGridDb;

impl Provider for SteamGridDb {
    fn name(&self) -> &'static str {
        "steamgriddb"
    }

    fn enabled(&self, secrets: &std::collections::BTreeMap<String, String>) -> bool {
        secrets.contains_key("sgdb-api-key")
    }

    fn enrich(
        &self,
        secrets: &std::collections::BTreeMap<String, String>,
        card: &GameCard,
    ) -> Result<Option<Enrichment>, Error> {
        let Some(key) = secrets.get("sgdb-api-key") else {
            return Ok(None);
        };

        let term = percent_encode(&card.title);
        let url = format!("https://www.steamgriddb.com/api/v2/search/autocomplete/{term}");
        let json = authenticated_get(&url, key)?;
        let resp: SearchResponse = serde_json::from_str(&json)
            .map_err(|e| Error::Runtime(format!("steamgriddb search json: {e}")))?;

        // Pick the closest name match; SteamGridDB autocomplete is usually good.
        if resp.data.is_empty() {
            return Ok(None);
        }

        // We do not store deep metadata from SteamGridDB; the value is artwork.
        Ok(Some(Enrichment::default()))
    }

    fn artwork(
        &self,
        secrets: &std::collections::BTreeMap<String, String>,
        card: &GameCard,
        kind: ArtworkKind,
    ) -> Result<Option<Vec<u8>>, Error> {
        let Some(key) = secrets.get("sgdb-api-key") else {
            return Ok(None);
        };

        let term = percent_encode(&card.title);
        let search_url = format!("https://www.steamgriddb.com/api/v2/search/autocomplete/{term}");
        let json = authenticated_get(&search_url, key)?;
        let resp: SearchResponse = serde_json::from_str(&json)
            .map_err(|e| Error::Runtime(format!("steamgriddb search json: {e}")))?;

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
        let json = authenticated_get(&url, key)?;
        let art: ArtResponse = serde_json::from_str(&json)
            .map_err(|e| Error::Runtime(format!("steamgriddb {endpoint} json: {e}")))?;

        for item in art.data {
            if item.url.is_empty() {
                continue;
            }
            // SteamGridDB serves PNG/JPEG; download binary bytes.
            match scraper::download_stream(&item.url) {
                Ok((_, mut reader)) => {
                    let mut bytes = Vec::new();
                    if reader.read_to_end(&mut bytes).is_ok() && !bytes.is_empty() {
                        return Ok(Some(bytes));
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
        .timeout_connect(Some(std::time::Duration::from_secs(15)))
        .timeout_recv_response(Some(std::time::Duration::from_secs(15)))
        .timeout_recv_body(Some(std::time::Duration::from_secs(15)))
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
    data: Vec<ArtItem>,
}

#[derive(Debug, Deserialize)]
struct ArtItem {
    url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_name_and_enabled_requires_key() {
        let p = SteamGridDb;
        assert_eq!(p.name(), "steamgriddb");
        let mut secrets = std::collections::BTreeMap::new();
        assert!(!p.enabled(&secrets));
        secrets.insert("sgdb-api-key".to_string(), "x".to_string());
        assert!(p.enabled(&secrets));
    }
}
