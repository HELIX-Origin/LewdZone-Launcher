//! IndieDB provider: description and images.
//!
//! No API key required. Uses the public game listing search + game pages.

use ::scraper::{Html, Selector};

use crate::core::models::GameCard;
use crate::core::Error;
use crate::scraper;

use super::{ArtworkKind, Enrichment, Provider};

pub struct IndieDb;

impl Provider for IndieDb {
    fn name(&self) -> &'static str {
        "indiedb"
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

        let description = doc
            .select(
                &Selector::parse("#articles .article .text")
                    .unwrap_or_else(|_| Selector::parse("xxx").unwrap()),
            )
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                doc.select(
                    &Selector::parse("meta[name=\"description\"]")
                        .unwrap_or_else(|_| Selector::parse("xxx").unwrap()),
                )
                .next()
                .and_then(|el| el.value().attr("content").map(|s| s.trim().to_string()))
                .filter(|s| !s.is_empty())
            });

        let developer = doc
            .select(
                &Selector::parse(".profileinfos .row a")
                    .unwrap_or_else(|_| Selector::parse("xxx").unwrap()),
            )
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty());

        let screenshots: Vec<String> = doc
            .select(
                &Selector::parse(".media a").unwrap_or_else(|_| Selector::parse("xxx").unwrap()),
            )
            .filter_map(|el| el.value().attr("href").map(|s| s.to_string()))
            .filter(|s| s.contains("/images/"))
            .take(6)
            .collect();

        Ok(Some(Enrichment {
            description,
            developer,
            rating: None,
            tags: Vec::new(),
            screenshots,
            genres: Vec::new(),
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

        let image = doc
            .select(
                &Selector::parse(".media a img")
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

        let Some(url) = image else {
            return Ok(None);
        };
        let url = if url.starts_with("//") {
            format!("https:{url}")
        } else {
            url
        };
        download_bytes(&url)
    }
}

fn search_first_result(title: &str) -> Result<Option<String>, Error> {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    let term = utf8_percent_encode(title, NON_ALPHANUMERIC).to_string();
    let url = format!("https://www.indiedb.com/games?filter=t&kw={term}&page=1");
    let html = scraper::fetch(&url)?;
    let doc = Html::parse_document(&html);

    Ok(doc
        .select(
            &Selector::parse(".row .four .thumb a")
                .unwrap_or_else(|_| Selector::parse("xxx").unwrap()),
        )
        .next()
        .and_then(|el| {
            el.value().attr("href").map(|s| {
                if s.starts_with("http://") || s.starts_with("https://") {
                    s.to_string()
                } else {
                    format!("https://www.indiedb.com{s}")
                }
            })
        }))
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
            eprintln!("[indiedb] failed to download {}: {}", url, e);
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_name_and_always_enabled() {
        let p = IndieDb;
        assert_eq!(p.name(), "indiedb");
        assert!(p.enabled(&std::collections::BTreeMap::new()));
    }
}
