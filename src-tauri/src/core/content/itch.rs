//! itch.io provider: description, developer, tags, screenshots, and cover.
//!
//! No API key required. Uses public search and game page scraper.

use ::scraper::{Html, Selector};
use std::collections::BTreeMap;
use std::io::Read;

use crate::core::models::GameCard;
use crate::core::Error;
use crate::scraper;

use super::{ArtworkKind, Enrichment, Provider};

pub struct Itch;

impl Provider for Itch {
    fn name(&self) -> &'static str {
        "itch"
    }

    fn enabled(&self, _secrets: &BTreeMap<String, String>) -> bool {
        true
    }

    fn enrich(
        &self,
        _secrets: &BTreeMap<String, String>,
        card: &GameCard,
    ) -> Result<Option<Enrichment>, Error> {
        let Some(link) = search_first_result(&card.title)? else {
            return Ok(None);
        };
        let html = match scraper::fetch(&link) {
            Ok(h) => h,
            Err(e) => {
                crate::core::logging::debug("itch", &format!("fetch failed for {link}: {e}"));
                return Ok(None);
            }
        };
        let doc = Html::parse_document(&html);

        let description = select_text(&doc, ".formatted_description");
        let developer = select_text(&doc, ".game_author a");
        let mut screenshots: Vec<String> = doc
            .select(
                &Selector::parse(".screenshot_list a")
                    .unwrap_or_else(|_| Selector::parse("xxx").unwrap()),
            )
            .filter_map(|el| el.value().attr("href").map(|s| s.to_string()))
            .take(10)
            .collect();

        // If no screenshots via anchor links, check img src in screenshot list
        if screenshots.is_empty() {
            screenshots = doc
                .select(
                    &Selector::parse(".screenshot_list img")
                        .unwrap_or_else(|_| Selector::parse("xxx").unwrap()),
                )
                .filter_map(|el| el.value().attr("src").map(|s| s.to_string()))
                .take(10)
                .collect();
        }

        let tags: Vec<String> = doc
            .select(&Selector::parse(".tag").unwrap_or_else(|_| Selector::parse("xxx").unwrap()))
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
            .take(8)
            .collect();

        Ok(Some(Enrichment {
            description,
            developer,
            rating: None,
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
        if !matches!(kind, ArtworkKind::Cover) {
            return Ok(None);
        }
        let Some(link) = search_first_result(&card.title)? else {
            return Ok(None);
        };
        let html = match scraper::fetch(&link) {
            Ok(h) => h,
            Err(_) => return Ok(None),
        };
        let doc = Html::parse_document(&html);

        let cover = doc
            .select(
                &Selector::parse(".header .image img")
                    .unwrap_or_else(|_| Selector::parse("xxx").unwrap()),
            )
            .next()
            .and_then(|el| el.value().attr("src").map(|s| s.to_string()))
            .or_else(|| {
                doc.select(
                    &Selector::parse("meta[property=\"og:image\"]")
                        .unwrap_or_else(|_| Selector::parse("xxx").unwrap()),
                )
                .next()
                .and_then(|el| el.value().attr("content").map(|s| s.to_string()))
            });

        let Some(url) = cover else {
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

fn search_first_result(title: &str) -> Result<Option<String>, Error> {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    let cleaned = clean_title(title);
    if cleaned.is_empty() {
        return Ok(None);
    }
    let term = utf8_percent_encode(&cleaned, NON_ALPHANUMERIC).to_string();
    let url = format!("https://itch.io/search?q={term}");
    let html = match scraper::fetch(&url) {
        Ok(h) => h,
        Err(e) => {
            crate::core::logging::debug("itch", &format!("search error for '{cleaned}': {e}"));
            return Ok(None);
        }
    };
    let doc = Html::parse_document(&html);

    Ok(doc
        .select(&Selector::parse(".game_link").unwrap_or_else(|_| Selector::parse("xxx").unwrap()))
        .next()
        .and_then(|el| {
            el.value().attr("href").map(|s| {
                if s.starts_with("http://") || s.starts_with("https://") {
                    s.to_string()
                } else {
                    format!("https://itch.io{s}")
                }
            })
        }))
}

fn select_text(doc: &Html, sel: &str) -> Option<String> {
    let selector = Selector::parse(sel).unwrap_or_else(|_| Selector::parse("xxx").unwrap());
    doc.select(&selector)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .filter(|s| !s.is_empty())
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
            crate::core::logging::debug("itch", &format!("download failed for {url}: {e}"));
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_name_and_always_enabled() {
        let p = Itch;
        assert_eq!(p.name(), "itch");
        assert!(p.enabled(&BTreeMap::new()));
    }
}
