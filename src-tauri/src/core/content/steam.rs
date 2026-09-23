//! Steam Storefront provider: description, developer, screenshots, capsule art.
//!
//! No API key required. Uses the public storesearch + appdetails endpoints.

use serde::Deserialize;

use crate::core::models::GameCard;
use crate::core::Error;
use crate::scraper;

use super::{ArtworkKind, Enrichment, Provider};

pub struct Steam;

impl Provider for Steam {
    fn name(&self) -> &'static str {
        "steam"
    }

    fn enabled(&self, _secrets: &std::collections::BTreeMap<String, String>) -> bool {
        true
    }

    fn enrich(
        &self,
        _secrets: &std::collections::BTreeMap<String, String>,
        card: &GameCard,
    ) -> Result<Option<Enrichment>, Error> {
        let Some(app_id) = search_app_id(&card.title)? else {
            return Ok(None);
        };
        let Some(details) = app_details(app_id)? else {
            return Ok(None);
        };

        let description = details.short_description.filter(|s| !s.is_empty());
        let developer = details.developers.into_iter().next();
        let screenshots: Vec<String> = details
            .screenshots
            .into_iter()
            .map(|s| s.path_full)
            .collect();

        Ok(Some(Enrichment {
            description,
            developer,
            rating: None,
            tags: Vec::new(),
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
        let Some(app_id) = search_app_id(&card.title)? else {
            return Ok(None);
        };
        let Some(details) = app_details(app_id)? else {
            return Ok(None);
        };
        let Some(url) = details.header_image else {
            return Ok(None);
        };
        download_bytes(&url)
    }
}

fn search_app_id(title: &str) -> Result<Option<u64>, Error> {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    let term = utf8_percent_encode(title, NON_ALPHANUMERIC).to_string();
    let url = format!("https://store.steampowered.com/api/storesearch/?term={term}&cc=US&l=en");

    let json = scraper::fetch(&url)?;
    let resp: StoreSearchResponse = serde_json::from_str(&json)
        .map_err(|e| Error::Runtime(format!("steam search json: {e}")))?;

    Ok(resp.items.into_iter().next().map(|item| item.id))
}

fn app_details(app_id: u64) -> Result<Option<AppDetails>, Error> {
    let url = format!("https://store.steampowered.com/api/appdetails?appids={app_id}&cc=US&l=en");
    let json = scraper::fetch(&url)?;
    let mut resp: AppDetailsResponse = serde_json::from_str(&json)
        .map_err(|e| Error::Runtime(format!("steam appdetails json: {e}")))?;

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
            eprintln!("[steam] failed to download {}: {}", url, e);
            Ok(None)
        }
    }
}

#[derive(Debug, Deserialize)]
struct StoreSearchResponse {
    items: Vec<StoreSearchItem>,
}

#[derive(Debug, Deserialize)]
struct StoreSearchItem {
    id: u64,
    name: String,
}

#[derive(Debug, Deserialize)]
struct AppDetailsResponse(std::collections::HashMap<u64, AppDetailsWrapper>);

#[derive(Debug, Deserialize)]
struct AppDetailsWrapper {
    data: Option<AppDetails>,
}

#[derive(Debug, Deserialize)]
struct AppDetails {
    short_description: Option<String>,
    developers: Vec<String>,
    screenshots: Vec<SteamScreenshot>,
    header_image: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SteamScreenshot {
    path_full: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_name_and_always_enabled() {
        let p = Steam;
        assert_eq!(p.name(), "steam");
        assert!(p.enabled(&std::collections::BTreeMap::new()));
    }
}
