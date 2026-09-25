//! Steam Storefront provider: description, developer, screenshots, background, and capsule art.
//!
//! No API key required. Uses public storesearch and appdetails endpoints.

use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::io::Read;

use crate::core::models::GameCard;
use crate::core::Error;
use crate::scraper;

use super::{ArtworkKind, Enrichment, Provider};

pub struct Steam;

impl Provider for Steam {
    fn name(&self) -> &'static str {
        "steam"
    }

    fn enabled(&self, _secrets: &BTreeMap<String, String>) -> bool {
        true
    }

    fn enrich(
        &self,
        _secrets: &BTreeMap<String, String>,
        card: &GameCard,
    ) -> Result<Option<Enrichment>, Error> {
        let Some(app_id) = search_app_id(&card.title)? else {
            return Ok(None);
        };
        let Some(details) = app_details(app_id)? else {
            return Ok(None);
        };

        let description = details.short_description.filter(|s| !s.trim().is_empty());
        let developer = details.developers.into_iter().next();
        let genres: Vec<String> = details
            .genres
            .into_iter()
            .map(|g| g.description)
            .filter(|g| !g.is_empty())
            .collect();

        let mut screenshots: Vec<String> = details
            .screenshots
            .into_iter()
            .map(|s| s.path_full)
            .filter(|s| !s.is_empty())
            .take(10)
            .collect();

        // If background is available and not already in screenshots, prepend/append
        if let Some(bg) = details.background_raw.or(details.background) {
            if !bg.is_empty() && !screenshots.contains(&bg) && screenshots.len() < 10 {
                screenshots.push(bg);
            }
        }

        let status = if genres
            .iter()
            .any(|g| g.to_lowercase().contains("early access"))
            || details.release_date.as_ref().is_some_and(|r| r.coming_soon)
        {
            Some("Ongoing".to_string())
        } else if details.release_date.is_some() {
            Some("Finished".to_string())
        } else {
            None
        };

        Ok(Some(Enrichment {
            description,
            developer,
            rating: None,
            status,
            tags: Vec::new(),
            genres,
            screenshots,
        }))
    }

    fn artwork(
        &self,
        _secrets: &BTreeMap<String, String>,
        card: &GameCard,
        kind: ArtworkKind,
    ) -> Result<Option<(String, Vec<u8>)>, Error> {
        let Some(app_id) = search_app_id(&card.title)? else {
            return Ok(None);
        };
        let Some(details) = app_details(app_id)? else {
            return Ok(None);
        };

        let target_url = match kind {
            ArtworkKind::Cover => details.header_image,
            ArtworkKind::Background => details.background_raw.or(details.background),
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

fn search_app_id(title: &str) -> Result<Option<u64>, Error> {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    let cleaned = clean_title(title);
    if cleaned.is_empty() {
        return Ok(None);
    }
    let term = utf8_percent_encode(&cleaned, NON_ALPHANUMERIC).to_string();
    let url = format!("https://store.steampowered.com/api/storesearch/?term={term}&cc=US&l=en");

    let json = match scraper::fetch(&url) {
        Ok(j) => j,
        Err(e) => {
            crate::core::logging::debug(
                "steam",
                &format!("storesearch error for '{cleaned}': {e}"),
            );
            return Ok(None);
        }
    };
    let resp: StoreSearchResponse = match serde_json::from_str(&json) {
        Ok(r) => r,
        Err(e) => {
            crate::core::logging::debug("steam", &format!("storesearch json parse error: {e}"));
            return Ok(None);
        }
    };

    Ok(resp.items.into_iter().next().map(|item| item.id))
}

fn app_details(app_id: u64) -> Result<Option<AppDetails>, Error> {
    let url = format!("https://store.steampowered.com/api/appdetails?appids={app_id}&cc=US&l=en");
    let json = match scraper::fetch(&url) {
        Ok(j) => j,
        Err(e) => {
            crate::core::logging::debug("steam", &format!("appdetails error for {app_id}: {e}"));
            return Ok(None);
        }
    };
    let mut resp: AppDetailsResponse = match serde_json::from_str(&json) {
        Ok(r) => r,
        Err(e) => {
            crate::core::logging::debug("steam", &format!("appdetails json parse error: {e}"));
            return Ok(None);
        }
    };

    Ok(resp.0.remove(&app_id).and_then(|wrapper| wrapper.data))
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
            crate::core::logging::debug("steam", &format!("download failed for {url}: {e}"));
            Ok(None)
        }
    }
}

#[derive(Debug, Deserialize)]
struct StoreSearchResponse {
    #[serde(default)]
    items: Vec<StoreSearchItem>,
}

#[derive(Debug, Deserialize)]
struct StoreSearchItem {
    id: u64,
}

#[derive(Debug, Deserialize)]
struct AppDetailsResponse(HashMap<u64, AppDetailsWrapper>);

#[derive(Debug, Deserialize)]
struct AppDetailsWrapper {
    data: Option<AppDetails>,
}

#[derive(Debug, Deserialize)]
struct AppDetails {
    #[serde(default)]
    short_description: Option<String>,
    #[serde(default)]
    developers: Vec<String>,
    #[serde(default)]
    genres: Vec<SteamGenre>,
    #[serde(default)]
    screenshots: Vec<SteamScreenshot>,
    #[serde(default)]
    header_image: Option<String>,
    #[serde(default)]
    background: Option<String>,
    #[serde(default)]
    background_raw: Option<String>,
    #[serde(default)]
    release_date: Option<SteamReleaseDate>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SteamReleaseDate {
    #[serde(default)]
    coming_soon: bool,
    #[serde(default)]
    date: String,
}

#[derive(Debug, Deserialize)]
struct SteamGenre {
    #[serde(default)]
    description: String,
}

#[derive(Debug, Deserialize)]
struct SteamScreenshot {
    #[serde(default)]
    path_full: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_title_for_steam() {
        assert_eq!(clean_title("[3DCG] Subverse - v1.0"), "Subverse");
        assert_eq!(clean_title("Wild Life"), "Wild Life");
    }

    #[test]
    fn provider_name_and_always_enabled() {
        let p = Steam;
        assert_eq!(p.name(), "steam");
        assert!(p.enabled(&BTreeMap::new()));
    }
}
