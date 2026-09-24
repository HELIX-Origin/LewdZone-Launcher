//! VNDB (Kana) provider: description, developer, rating, tags, screenshots, and cover.
//!
//! No API key required. API docs: https://api.vndb.org/kana

use serde::Deserialize;
use serde_json::json;
use std::collections::BTreeMap;
use std::io::Read;

use crate::core::models::GameCard;
use crate::core::Error;
use crate::scraper;

use super::{ArtworkKind, Enrichment, Provider};

pub struct Vndb;

impl Provider for Vndb {
    fn name(&self) -> &'static str {
        "vndb"
    }

    fn enabled(&self, _secrets: &BTreeMap<String, String>) -> bool {
        true
    }

    fn enrich(
        &self,
        _secrets: &BTreeMap<String, String>,
        card: &GameCard,
    ) -> Result<Option<Enrichment>, Error> {
        let Some(vn) = search_vn(&card.title)? else {
            return Ok(None);
        };

        let description = if !vn.description.is_empty() {
            Some(clean_bbcode(&vn.description))
        } else {
            None
        };
        let developer = vn
            .developers
            .into_iter()
            .next()
            .map(|p| p.name)
            .or_else(|| vn.producers.into_iter().next().map(|p| p.name));
        let rating = vn.rating.map(|r| (r / 2.0).clamp(0.0, 5.0)); // 0-10 -> 0-5 stars
        let tags: Vec<String> = vn.tags.into_iter().take(8).map(|t| t.name).collect();
        let screenshots: Vec<String> = vn
            .screenshots
            .into_iter()
            .filter_map(|s| {
                if !s.url.is_empty() {
                    Some(s.url)
                } else if !s.thumbnail.is_empty() {
                    Some(s.thumbnail.replace("/sf.t/", "/sf/").replace("/s/", "/"))
                } else {
                    None
                }
            })
            .take(10)
            .collect();

        Ok(Some(Enrichment {
            description,
            developer,
            rating,
            tags,
            genres: Vec::new(),
            screenshots,
        }))
    }

    fn artwork(
        &self,
        _secrets: &BTreeMap<String, String>,
        card: &GameCard,
        kind: ArtworkKind,
    ) -> Result<Option<(String, Vec<u8>)>, Error> {
        let Some(vn) = search_vn(&card.title)? else {
            return Ok(None);
        };

        let target_url = match kind {
            ArtworkKind::Cover => vn.image.map(|img| img.url).filter(|u| !u.is_empty()),
            ArtworkKind::Background => vn.screenshots.into_iter().find_map(|s| {
                if !s.url.is_empty() {
                    Some(s.url)
                } else {
                    None
                }
            }),
            ArtworkKind::Icon => None,
        };

        let Some(url) = target_url else {
            return Ok(None);
        };

        let ext = if url.contains(".png") {
            "png".to_string()
        } else if url.contains(".webp") {
            "webp".to_string()
        } else {
            "jpg".to_string()
        };

        match download_bytes(&url)? {
            Some(bytes) => Ok(Some((ext, bytes))),
            None => Ok(None),
        }
    }
}

pub fn clean_title(title: &str) -> String {
    let t = if title.starts_with('[') {
        if let Some((_, rest)) = title.split_once(']') {
            rest
        } else {
            title
        }
    } else {
        title
    };
    let t = t.split(" - ").next().unwrap_or(t);
    let t = t.split(" [").next().unwrap_or(t);
    let t = t.split(" (").next().unwrap_or(t);
    t.trim().to_string()
}

fn clean_bbcode(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut in_tag = false;
    for c in raw.chars() {
        if c == '[' {
            in_tag = true;
        } else if c == ']' {
            in_tag = false;
        } else if !in_tag {
            out.push(c);
        }
    }
    out.replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .trim()
        .to_string()
}

fn search_vn(title: &str) -> Result<Option<Vn>, Error> {
    let term = clean_title(title);
    if term.is_empty() {
        return Ok(None);
    }

    let query = json!({
        "filters": ["search", "=", term],
        "fields": "id,title,description,rating,image{url},screenshots{url,thumbnail},developers{name},producers{name},tags{name}",
        "results": 1,
    });
    let body = query.to_string();

    let agent = ureq::Agent::config_builder()
        .timeout_connect(Some(std::time::Duration::from_secs(10)))
        .timeout_recv_response(Some(std::time::Duration::from_secs(10)))
        .timeout_recv_body(Some(std::time::Duration::from_secs(10)))
        .build()
        .new_agent();

    let resp = match agent
        .post("https://api.vndb.org/kana/vn")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .send(&body)
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[vndb] search request error for '{term}': {e}");
            return Ok(None);
        }
    };

    if resp.status() != 200 {
        return Ok(None);
    }

    let text = match resp.into_body().read_to_string() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("[vndb] body read error: {e}");
            return Ok(None);
        }
    };

    let result: SearchResponse = match serde_json::from_str(&text) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[vndb] json parse error: {e}");
            return Ok(None);
        }
    };

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
            eprintln!("[vndb] download failed for {url}: {e}");
            Ok(None)
        }
    }
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    #[serde(default)]
    results: Vec<Vn>,
}

#[derive(Debug, Deserialize)]
struct Vn {
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub rating: Option<f32>,
    #[serde(default)]
    pub image: Option<VnImage>,
    #[serde(default)]
    pub screenshots: Vec<VnScreenshot>,
    #[serde(default)]
    pub developers: Vec<NamedItem>,
    #[serde(default)]
    pub producers: Vec<NamedItem>,
    #[serde(default)]
    pub tags: Vec<NamedItem>,
}

#[derive(Debug, Deserialize)]
struct VnImage {
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Deserialize)]
struct VnScreenshot {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub thumbnail: String,
}

#[derive(Debug, Deserialize)]
struct NamedItem {
    #[serde(default)]
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_title_strips_brackets_and_subtitles() {
        assert_eq!(
            clean_title("[RPGM] Treasure of Nadia - v1.0117"),
            "Treasure of Nadia"
        );
        assert_eq!(clean_title("Wild Life [Ongoing]"), "Wild Life");
        assert_eq!(
            clean_title("Doki Doki Literature Club!"),
            "Doki Doki Literature Club!"
        );
    }

    #[test]
    fn provider_name_and_always_enabled() {
        let p = Vndb;
        assert_eq!(p.name(), "vndb");
        assert!(p.enabled(&BTreeMap::new()));
    }
}
