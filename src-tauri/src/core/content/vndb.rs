//! VNDB (Kana) provider: description, developer, tags, screenshots, cover.
//!
//! No API key required. API docs: https://api.vndb.org/kana

use serde::Deserialize;
use serde_json::json;

use crate::core::models::GameCard;
use crate::core::Error;
use crate::scraper;

use super::{ArtworkKind, Enrichment, Provider};

pub struct Vndb;

impl Provider for Vndb {
    fn name(&self) -> &'static str {
        "vndb"
    }

    fn enabled(&self, _secrets: &std::collections::BTreeMap<String, String>) -> bool {
        true
    }

    fn enrich(
        &self,
        _secrets: &std::collections::BTreeMap<String, String>,
        card: &GameCard,
    ) -> Result<Option<Enrichment>, Error> {
        let Some(vn) = search_vn(&card.title)? else {
            return Ok(None);
        };

        let description = (!vn.description.is_empty()).then(|| vn.description);
        let developer = vn
            .developers
            .into_iter()
            .next()
            .map(|p| p.name)
            .or_else(|| vn.producers.into_iter().next().map(|p| p.name));
        let rating = vn.rating;
        let tags: Vec<String> = vn.tags.into_iter().take(8).collect();
        let screenshots: Vec<String> = vn
            .screenshots
            .into_iter()
            .filter_map(|s| s.thumbnail.replace("/s/", "/"))
            .collect();

        Ok(Some(Enrichment {
            description,
            developer,
            rating,
            tags,
            screenshots,
        }))
    }

    fn artwork(
        &self,
        _secrets: &std::collections::BTreeMap<String, String>,
        card: &GameCard,
        kind: ArtworkKind,
    ) -> Result<Option<Vec<u8>>, Error> {
        if !matches!(kind, ArtworkKind::Cover) {
            return Ok(None);
        }
        let Some(vn) = search_vn(&card.title)? else {
            return Ok(None);
        };
        let Some(url) = vn.image else {
            return Ok(None);
        };
        download_bytes(&url)
    }
}

fn search_vn(title: &str) -> Result<Option<Vn>, Error> {
    let query = json!({
        "filters": ["search", "=", title],
        "fields": "id,title,description,rating,image,screenshots{thumbnail},developers{name},producers{name},tags{name}",
        "results": 1,
    });
    let body = query.to_string();

    let agent = ureq::Agent::config_builder()
        .timeout_connect(Some(std::time::Duration::from_secs(15)))
        .timeout_recv_response(Some(std::time::Duration::from_secs(15)))
        .timeout_recv_body(Some(std::time::Duration::from_secs(15)))
        .build()
        .new_agent();

    let resp = agent
        .post("https://api.vndb.org/kana/vn")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .send(&body)
        .map_err(|e| Error::Network(format!("vndb search: {e}")))?;

    if resp.status() != 200 {
        return Err(Error::Network(format!(
            "vndb search: HTTP {}",
            resp.status()
        )));
    }

    let text = resp
        .into_body()
        .read_to_string()
        .map_err(|e| Error::Network(format!("vndb search body: {e}")))?;
    let result: SearchResponse = serde_json::from_str(&text)
        .map_err(|e| Error::Runtime(format!("vndb search json: {e}")))?;

    Ok(result.results.into_iter().next())
}

fn download_bytes(url: &str) -> Result<Option<Vec<u8>>, Error> {
    match scraper::download_stream(url) {
        Ok((_, mut reader)) => {
            let mut bytes = Vec::new();
            if reader.read_to_end(&mut bytes).is_ok() && !bytes.is_empty() {
                Ok(Some(bytes))
            } else {
                Ok(None)
            }
        }
        Err(e) => {
            eprintln!("[vndb] failed to download {}: {}", url, e);
            Ok(None)
        }
    }
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    results: Vec<Vn>,
}

#[derive(Debug, Deserialize)]
struct Vn {
    id: String,
    title: String,
    description: String,
    rating: Option<f32>,
    image: Option<String>,
    screenshots: Vec<Screenshot>,
    developers: Vec<Producer>,
    producers: Vec<Producer>,
    tags: Vec<Tag>,
}

#[derive(Debug, Deserialize)]
struct Screenshot {
    thumbnail: String,
}

#[derive(Debug, Deserialize)]
struct Producer {
    name: String,
}

#[derive(Debug, Deserialize)]
struct Tag {
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_name_and_always_enabled() {
        let p = Vndb;
        assert_eq!(p.name(), "vndb");
        assert!(p.enabled(&std::collections::BTreeMap::new()));
    }
}
