//! itch.io provider: description, developer, platforms, screenshots, cover.
//!
//! No API key required. Uses the public search + game pages. If the user has
//! the itch.io desktop app installed, authentication can be added later.

use ::scraper::{Html, Selector};

use crate::core::models::GameCard;
use crate::core::Error;
use crate::scraper;

use super::{ArtworkKind, Enrichment, Provider};

pub struct Itch;

impl Provider for Itch {
    fn name(&self) -> &'static str {
        "itch"
    }

    fn enabled(&self, _secrets: &std::collections::BTreeMap<String, String>) -> bool {
        true
    }

    fn enrich(
        &self,
        _secrets: &std::collections::BTreeMap<String, String>,
        card: &GameCard,
    ) -> Result<Option<Enrichment>, Error> {
        let Some(link) = search_first_result(&card.title)? else {
            return Ok(None);
        };
        let html = scraper::fetch(&link)?;
        let doc = Html::parse_document(&html);

        let description = select_text(&doc, ".formatted_description");
        let developer = select_text(&doc, ".game_author a");
        let screenshots: Vec<String> = doc
            .select(
                &Selector::parse(".screenshot_list a")
                    .unwrap_or_else(|_| Selector::parse("xxx").unwrap()),
            )
            .filter_map(|el| el.value().attr("href").map(|s| s.to_string()))
            .collect();
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
        let Some(link) = search_first_result(&card.title)? else {
            return Ok(None);
        };
        let html = scraper::fetch(&link)?;
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
        download_bytes(&url)
    }
}

fn search_first_result(title: &str) -> Result<Option<String>, Error> {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    let term = utf8_percent_encode(title, NON_ALPHANUMERIC).to_string();
    let url = format!("https://itch.io/search?q={term}");
    let html = scraper::fetch(&url)?;
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
            eprintln!("[itch] failed to download {}: {}", url, e);
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
        assert!(p.enabled(&std::collections::BTreeMap::new()));
    }
}
