//! IGDB provider: description, genres, screenshots, cover art.
//!
//! Requires Twitch Client-ID and Client-Secret stored in SQLite secrets
//! (`igdb-client-id`, `igdb-client-secret`). IGDB docs:
//! https://api-docs.igdb.com/

use serde::Deserialize;

use crate::core::models::GameCard;
use crate::core::Error;
use crate::scraper;

use super::{ArtworkKind, Enrichment, Provider};

pub struct Igdb;

impl Provider for Igdb {
    fn name(&self) -> &'static str {
        "igdb"
    }

    fn enabled(&self, secrets: &std::collections::BTreeMap<String, String>) -> bool {
        secrets.contains_key("igdb-client-id") && secrets.contains_key("igdb-client-secret")
    }

    fn enrich(
        &self,
        secrets: &std::collections::BTreeMap<String, String>,
        card: &GameCard,
    ) -> Result<Option<Enrichment>, Error> {
        let token = fetch_token(secrets)?;
        let games = query_games(&token, secrets, &card.title)?;
        let Some(game) = games.into_iter().next() else {
            return Ok(None);
        };

        let description = game.summary.filter(|s| !s.is_empty());
        let tags: Vec<String> = game
            .genres
            .unwrap_or_default()
            .into_iter()
            .map(|g| g.name)
            .collect();
        let screenshots: Vec<String> = game
            .screenshots
            .unwrap_or_default()
            .into_iter()
            .filter_map(|s| expand_image_url(&s.url))
            .collect();

        Ok(Some(Enrichment {
            description,
            developer: None,
            rating: None,
            tags,
            screenshots,
        }))
    }

    fn artwork(
        &self,
        secrets: &std::collections::BTreeMap<String, String>,
        card: &GameCard,
        kind: ArtworkKind,
    ) -> Result<Option<Vec<u8>>, Error> {
        if !matches!(kind, ArtworkKind::Cover) {
            return Ok(None);
        }
        let token = fetch_token(secrets)?;
        let games = query_games(&token, secrets, &card.title)?;
        let Some(game) = games.into_iter().next() else {
            return Ok(None);
        };
        let Some(cover) = game.cover else {
            return Ok(None);
        };
        let Some(url) = expand_image_url(&cover.url) else {
            return Ok(None);
        };
        download_bytes(&url)
    }
}

fn fetch_token(secrets: &std::collections::BTreeMap<String, String>) -> Result<String, Error> {
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
        .call()
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
    secrets: &std::collections::BTreeMap<String, String>,
    title: &str,
) -> Result<Vec<IgdbGame>, Error> {
    let client_id = secrets.get("igdb-client-id").cloned().unwrap_or_default();
    let body = format!(
        "search \"{}\"; fields name,summary,genres.name,screenshots.url,cover.url; limit 1;",
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

fn expand_image_url(url: &str) -> Option<String> {
    if url.starts_with("//") {
        Some(format!("https:{url}"))
    } else if url.starts_with("http://") || url.starts_with("https://") {
        Some(url.to_string())
    } else {
        None
    }
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
            eprintln!("[igdb] failed to download {}: {}", url, e);
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
    name: String,
    summary: Option<String>,
    genres: Option<Vec<Named>>,
    screenshots: Option<Vec<Image>>,
    cover: Option<Image>,
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
        let mut secrets = std::collections::BTreeMap::new();
        assert!(!p.enabled(&secrets));
        secrets.insert("igdb-client-id".to_string(), "x".to_string());
        assert!(!p.enabled(&secrets));
        secrets.insert("igdb-client-secret".to_string(), "y".to_string());
        assert!(p.enabled(&secrets));
    }

    #[test]
    fn expands_protocol_relative_image_urls() {
        assert_eq!(
            expand_image_url("//images.igdb.com/cover.jpg"),
            Some("https://images.igdb.com/cover.jpg".to_string())
        );
        assert_eq!(
            expand_image_url("https://example.com/cover.jpg"),
            Some("https://example.com/cover.jpg".to_string())
        );
    }
}
