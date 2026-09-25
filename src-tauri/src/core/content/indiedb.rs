//! IndieDB provider: description, developer, screenshots, and cover.
//!
//! No API key required. Uses public game listing search and game pages.

use ::scraper::{Html, Selector};
use std::collections::BTreeMap;
use std::io::Read;

use crate::core::models::GameCard;
use crate::core::Error;
use crate::scraper;

use super::{ArtworkKind, Enrichment, Provider};

pub struct IndieDb;

impl Provider for IndieDb {
    fn name(&self) -> &'static str {
        "indiedb"
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
        let Some(html) = fetch_html(&link) else {
            return Ok(None);
        };
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

        let mut screenshots: Vec<String> = doc
            .select(
                &Selector::parse(".media a").unwrap_or_else(|_| Selector::parse("xxx").unwrap()),
            )
            .filter_map(|el| el.value().attr("href").map(|s| s.to_string()))
            .filter(|s| s.contains("/images/"))
            .take(10)
            .collect();

        // If no screenshots in .media a, try images directly
        if screenshots.is_empty() {
            screenshots = doc
                .select(
                    &Selector::parse(".media img")
                        .unwrap_or_else(|_| Selector::parse("xxx").unwrap()),
                )
                .filter_map(|el| el.value().attr("src").map(|s| s.to_string()))
                .filter(|s| s.contains("/images/"))
                .take(10)
                .collect();
        }

        Ok(Some(Enrichment {
            description,
            developer,
            rating: None,
            status: None,
            tags: Vec::new(),
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
        let Some(html) = fetch_html(&link) else {
            return Ok(None);
        };
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

fn fetch_html(url: &str) -> Option<String> {
    let agent = ureq::Agent::config_builder()
        .timeout_connect(Some(std::time::Duration::from_secs(10)))
        .timeout_recv_response(Some(std::time::Duration::from_secs(10)))
        .timeout_recv_body(Some(std::time::Duration::from_secs(10)))
        .build()
        .new_agent();

    let resp = match agent
        .get(url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
        )
        .header(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        )
        .header("Accept-Language", "en-US,en;q=0.9")
        .call()
    {
        Ok(r) => r,
        Err(e) => {
            crate::core::logging::debug("indiedb", &format!("fetch error for {url}: {e}"));
            return None;
        }
    };

    if resp.status() != 200 {
        crate::core::logging::debug(
            "indiedb",
            &format!("returned status {} for {url}", resp.status()),
        );
        return None;
    }

    match resp.into_body().read_to_string() {
        Ok(t) => Some(t),
        Err(e) => {
            crate::core::logging::debug("indiedb", &format!("body read error: {e}"));
            None
        }
    }
}

fn search_first_result(title: &str) -> Result<Option<String>, Error> {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    let cleaned = clean_title(title);
    if cleaned.is_empty() {
        return Ok(None);
    }
    let term = utf8_percent_encode(&cleaned, NON_ALPHANUMERIC).to_string();
    let url = format!("https://www.indiedb.com/games?filter=t&kw={term}&page=1");
    let Some(html) = fetch_html(&url) else {
        return Ok(None);
    };
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
            crate::core::logging::debug("indiedb", &format!("download failed for {url}: {e}"));
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
        assert!(p.enabled(&BTreeMap::new()));
    }
}
