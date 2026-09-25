//! IGDB provider: description, genres, rating, screenshots, and cover art.
//!
//! Requires Twitch Client-ID and Client-Secret stored in SQLite secrets
//! (`igdb-client-id`, `igdb-client-secret`). IGDB docs:
//! https://api-docs.igdb.com/

use serde::Deserialize;
use std::collections::BTreeMap;
use std::io::Read;

use crate::core::models::GameCard;
use crate::core::Error;
use crate::scraper;

use super::{ArtworkKind, Enrichment, Provider};

pub struct Igdb;

impl Provider for Igdb {
    fn name(&self) -> &'static str {
        "igdb"
    }

    fn enabled(&self, secrets: &BTreeMap<String, String>) -> bool {
        secrets
            .get("igdb-client-id")
            .filter(|k| !k.trim().is_empty())
            .is_some()
            && secrets
                .get("igdb-client-secret")
                .filter(|k| !k.trim().is_empty())
                .is_some()
    }

    fn enrich(
        &self,
        secrets: &BTreeMap<String, String>,
        card: &GameCard,
    ) -> Result<Option<Enrichment>, Error> {
        let token = match fetch_token(secrets) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("[igdb] token fetch error: {e}");
                return Ok(None);
            }
        };

        let games = match query_games(&token, secrets, &card.title) {
            Ok(g) => g,
            Err(e) => {
                eprintln!("[igdb] game query error for '{}': {e}", card.title);
                return Ok(None);
            }
        };

        let Some(game) = games.into_iter().next() else {
            return Ok(None);
        };

        let description = game.summary.filter(|s| !s.is_empty());
        let genres: Vec<String> = game
            .genres
            .unwrap_or_default()
            .into_iter()
            .map(|g| g.name)
            .collect();
        let screenshots: Vec<String> = game
            .screenshots
            .unwrap_or_default()
            .into_iter()
            .filter_map(|s| expand_image_url(&s.url, "t_1080p"))
            .collect();
        let rating = game.rating.map(|r| (r / 20.0) as f32); // Convert 0-100 to 0-5 stars
        let status = game.status.and_then(|s| match s {
            0 | 5 => Some("Finished".to_string()),
            2 | 3 | 4 => Some("Ongoing".to_string()),
            6 => Some("Abandoned".to_string()),
            _ => None,
        });

        Ok(Some(Enrichment {
            description,
            developer: None,
            rating,
            status,
            tags: Vec::new(),
            genres,
            screenshots,
        }))
    }

    fn artwork(
        &self,
        secrets: &BTreeMap<String, String>,
        card: &GameCard,
        kind: ArtworkKind,
    ) -> Result<Option<(String, Vec<u8>)>, Error> {
        let token = match fetch_token(secrets) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("[igdb] token fetch error: {e}");
                return Ok(None);
            }
        };

        let games = match query_games(&token, secrets, &card.title) {
            Ok(g) => g,
            Err(e) => {
                eprintln!("[igdb] game query error for '{}': {e}", card.title);
                return Ok(None);
            }
        };

        let Some(game) = games.into_iter().next() else {
            return Ok(None);
        };

        let img_url = match kind {
            ArtworkKind::Cover => game
                .cover
                .and_then(|c| expand_image_url(&c.url, "t_cover_big")),
            ArtworkKind::Background => game
                .screenshots
                .unwrap_or_default()
                .into_iter()
                .find_map(|s| expand_image_url(&s.url, "t_1080p")),
            ArtworkKind::Icon => game.cover.and_then(|c| expand_image_url(&c.url, "t_thumb")),
        };

        let Some(url) = img_url else {
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

fn fetch_token(secrets: &BTreeMap<String, String>) -> Result<String, Error> {
    let client_id = secrets
        .get("igdb-client-id")
        .ok_or_else(|| Error::Usage("igdb-client-id secret missing".into()))?;
    let client_secret = secrets
        .get("igdb-client-secret")
        .ok_or_else(|| Error::Usage("igdb-client-secret secret missing".into()))?;

    let url = format!(
        "https://id.twitch.tv/oauth2/token?client_id={client_id}&client_secret={client_secret}&grant_type=client_credentials"
    );

    let agent = ureq::Agent::config_builder()
        .timeout_connect(Some(std::time::Duration::from_secs(15)))
        .timeout_recv_response(Some(std::time::Duration::from_secs(15)))
        .timeout_recv_body(Some(std::time::Duration::from_secs(15)))
        .build()
        .new_agent();

    let resp = agent
        .post(&url)
        .header("Accept", "application/json")
        .send_empty()
        .map_err(|e| Error::Network(format!("igdb token: {e}")))?;

    if resp.status() != 200 {
        return Err(Error::Network(format!(
            "igdb token: HTTP {}",
            resp.status()
        )));
    }

    let text = resp
        .into_body()
        .read_to_string()
        .map_err(|e| Error::Network(format!("igdb token body: {e}")))?;
    let token: TokenResponse =
        serde_json::from_str(&text).map_err(|e| Error::Runtime(format!("igdb token json: {e}")))?;

    Ok(token.access_token)
}

fn query_games(
    token: &str,
    secrets: &BTreeMap<String, String>,
    title: &str,
) -> Result<Vec<IgdbGame>, Error> {
    let client_id = secrets.get("igdb-client-id").cloned().unwrap_or_default();
    let body = format!(
        "search \"{}\"; fields name,summary,genres.name,screenshots.url,cover.url,rating,status; limit 1;",
        title.replace('"', "\\\"")
    );

    let agent = ureq::Agent::config_builder()
        .timeout_connect(Some(std::time::Duration::from_secs(15)))
        .timeout_recv_response(Some(std::time::Duration::from_secs(15)))
        .timeout_recv_body(Some(std::time::Duration::from_secs(15)))
        .build()
        .new_agent();

    let resp = agent
        .post("https://api.igdb.com/v4/games")
        .header("Client-ID", &client_id)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Accept", "application/json")
        .send(&body)
        .map_err(|e| Error::Network(format!("igdb games: {e}")))?;

    if resp.status() != 200 {
        return Err(Error::Network(format!(
            "igdb games: HTTP {}",
            resp.status()
        )));
    }

    let text = resp
        .into_body()
        .read_to_string()
        .map_err(|e| Error::Network(format!("igdb games body: {e}")))?;
    serde_json::from_str(&text).map_err(|e| Error::Runtime(format!("igdb games json: {e}")))
}

fn expand_image_url(url: &str, size: &str) -> Option<String> {
    let clean = if url.starts_with("//") {
        format!("https:{url}")
    } else if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        return None;
    };
    // Replace default t_thumb with requested size (e.g. t_cover_big or t_1080p)
    Some(clean.replace("t_thumb", size))
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
            eprintln!("[igdb] failed to download {url}: {e}");
            Ok(None)
        }
    }
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct IgdbGame {
    #[allow(dead_code)]
    name: String,
    summary: Option<String>,
    genres: Option<Vec<Named>>,
    screenshots: Option<Vec<Image>>,
    cover: Option<Image>,
    rating: Option<f64>,
    status: Option<u8>,
}

#[derive(Debug, Deserialize)]
struct Named {
    name: String,
}

#[derive(Debug, Deserialize)]
struct Image {
    url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_name_and_enabled_requires_both_secrets() {
        let p = Igdb;
        assert_eq!(p.name(), "igdb");
        let mut secrets = BTreeMap::new();
        assert!(!p.enabled(&secrets));
        secrets.insert("igdb-client-id".to_string(), "x".to_string());
        assert!(!p.enabled(&secrets));
        secrets.insert("igdb-client-secret".to_string(), "y".to_string());
        assert!(p.enabled(&secrets));
    }

    #[test]
    fn expands_protocol_relative_image_urls_and_sizes() {
        assert_eq!(
            expand_image_url(
                "//images.igdb.com/igdb/image/upload/t_thumb/co123.jpg",
                "t_cover_big"
            ),
            Some("https://images.igdb.com/igdb/image/upload/t_cover_big/co123.jpg".to_string())
        );
        assert_eq!(
            expand_image_url("https://example.com/t_thumb/co123.jpg", "t_1080p"),
            Some("https://example.com/t_1080p/co123.jpg".to_string())
        );
    }
}
