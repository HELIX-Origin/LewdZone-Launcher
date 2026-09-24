//! Content layer: cached artwork and game enrichment models.
//! External source providers have been removed (ADR-0006); LewdZone scraped
//! data is the sole authority for presentation and downloads.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::models::GameCard;
use crate::core::{paths, Context, Error};
use crate::db;

/// Kinds of artwork the launcher can cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtworkKind {
    Icon,
    Cover,
    Background,
}

impl std::fmt::Display for ArtworkKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArtworkKind::Icon => write!(f, "icon"),
            ArtworkKind::Cover => write!(f, "cover"),
            ArtworkKind::Background => write!(f, "background"),
        }
    }
}

/// Enrichment data for a single game.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Enrichment {
    pub description: Option<String>,
    pub developer: Option<String>,
    pub rating: Option<f32>,
    pub tags: Vec<String>,
    pub genres: Vec<String>,
    pub screenshots: Vec<String>,
}

/// Stable cache key for a game title. Lowercase, collapsed whitespace.
pub fn cache_key(title: &str) -> String {
    title
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

/// Return the directory where artwork files are cached.
pub fn artwork_dir() -> Result<PathBuf, Error> {
    paths::library_artwork_dir().ok_or_else(|| {
        Error::Runtime("cannot resolve artwork cache directory (no app data dir)".to_string())
    })
}

pub mod igdb;
pub mod indiedb;
pub mod itch;
pub mod steam;
pub mod steamgriddb;
pub mod vndb;

use std::collections::BTreeMap;
use std::io::Write;

/// Best-effort enrichment for a game. LewdZone scraped data is the authoritative baseline;
/// external providers (VNDB, Steam, itch.io, IndieDB, SteamGridDB, IGDB) supply missing or improved metadata.
pub fn enrich(ctx: &Context, card: &GameCard) -> Result<Enrichment, Error> {
    let mut merged = Enrichment {
        description: card.description.clone(),
        developer: card.developer.clone(),
        rating: None,
        tags: card.genres.clone(),
        genres: card.external_genres.clone(),
        screenshots: Vec::new(),
    };

    if let Some(ref thumb) = card.thumb_url {
        if !thumb.trim().is_empty() {
            merged.screenshots.push(thumb.clone());
        }
    }

    let secrets = load_secrets(ctx)?;
    for provider in providers() {
        if provider.enabled(&secrets) {
            match provider.enrich(&secrets, card) {
                Ok(Some(extra)) => {
                    if merged.description.is_none()
                        || merged
                            .description
                            .as_deref()
                            .unwrap_or("")
                            .trim()
                            .is_empty()
                    {
                        if let Some(d) = extra.description {
                            if !d.trim().is_empty() {
                                merged.description = Some(d);
                            }
                        }
                    }
                    if merged.developer.is_none()
                        || merged.developer.as_deref().unwrap_or("").trim().is_empty()
                    {
                        if let Some(dev) = extra.developer {
                            if !dev.trim().is_empty() {
                                merged.developer = Some(dev);
                            }
                        }
                    }
                    if merged.rating.is_none() {
                        merged.rating = extra.rating;
                    }
                    for g in extra.genres {
                        if !merged.genres.contains(&g) {
                            merged.genres.push(g);
                        }
                    }
                    for t in extra.tags {
                        if !merged.tags.contains(&t) {
                            merged.tags.push(t);
                        }
                    }
                    for s in extra.screenshots {
                        if !merged.screenshots.contains(&s) && merged.screenshots.len() < 10 {
                            merged.screenshots.push(s);
                        }
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    crate::core::logging::debug(
                        "content",
                        &format!("provider {} enrich error: {e}", provider.name()),
                    );
                }
            }
        }
    }

    Ok(merged)
}

/// A content provider.
pub trait Provider: Send + Sync {
    fn name(&self) -> &'static str;

    /// Whether this provider can run given the secrets available.
    fn enabled(&self, secrets: &BTreeMap<String, String>) -> bool;

    /// Try to enrich a game. Returns `Ok(None)` when the provider has nothing.
    fn enrich(
        &self,
        secrets: &BTreeMap<String, String>,
        card: &GameCard,
    ) -> Result<Option<Enrichment>, Error>;

    /// Try to fetch artwork bytes. Returns `Ok(None)` when nothing is found.
    fn artwork(
        &self,
        secrets: &BTreeMap<String, String>,
        card: &GameCard,
        kind: ArtworkKind,
    ) -> Result<Option<(String, Vec<u8>)>, Error>;
}

/// All registered providers.
pub fn providers() -> Vec<Box<dyn Provider>> {
    vec![
        Box::new(steamgriddb::SteamGridDb),
        Box::new(igdb::Igdb),
        Box::new(vndb::Vndb),
        Box::new(steam::Steam),
        Box::new(itch::Itch),
        Box::new(indiedb::IndieDb),
    ]
}

/// Load all secrets this layer cares about from SQLite.
pub fn load_secrets(ctx: &Context) -> Result<BTreeMap<String, String>, Error> {
    let keys = ["sgdb-api-key", "igdb-client-id", "igdb-client-secret"];
    let conn = db::open(&ctx.db_path)?;
    db::migrate(&conn)?;
    let mut out = BTreeMap::new();
    for key in keys {
        if let Some(value) = db::repo::secret_get(&conn, key)? {
            out.insert(key.to_string(), value);
        }
    }
    Ok(out)
}

/// Fetch (or return a cached) artwork file for a game. The returned path is
/// absolute and safe to use in `<img>`/CSS.
pub fn artwork(
    ctx: &Context,
    card: &GameCard,
    kind: ArtworkKind,
) -> Result<Option<PathBuf>, Error> {
    let key = cache_key(&card.title);
    let dir = artwork_dir()?;
    fs::create_dir_all(&dir)?;

    // 1. Check the database cache first.
    {
        let conn = db::open(&ctx.db_path)?;
        if let Some(row) = db::repo::artwork_cache_get(&conn, &key, &kind.to_string())? {
            let path = PathBuf::from(&row.file_path);
            if path.exists() {
                return Ok(Some(path));
            }
        }
    }

    // 2. Query enabled providers (e.g. SteamGridDB if API key is configured).
    let secrets = load_secrets(ctx)?;
    for provider in providers() {
        if !provider.enabled(&secrets) {
            continue;
        }
        match provider.artwork(&secrets, card, kind) {
            Ok(Some((_remote_url, bytes))) if !bytes.is_empty() => {
                let ext = image_ext(&bytes).unwrap_or("jpg");
                let file_name = format!("{key}-{kind}.{ext}");
                let path = dir.join(&file_name);
                let mut file = fs::File::create(&path)?;
                file.write_all(&bytes)?;

                let conn = db::open(&ctx.db_path)?;
                let _ = db::repo::artwork_cache_upsert(
                    &conn,
                    &key,
                    &kind.to_string(),
                    provider.name(),
                    &path.to_string_lossy(),
                );
                return Ok(Some(path));
            }
            Ok(_) => {}
            Err(e) => {
                crate::core::logging::debug(
                    "content",
                    &format!(
                        "{} artwork failed for '{}' ({:?}): {e}",
                        provider.name(),
                        card.title,
                        kind
                    ),
                );
            }
        }
    }

    Ok(None)
}

/// Guess an image extension from the first bytes.
#[allow(dead_code)]
fn image_ext(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("png")
    } else if bytes.starts_with(b"\xff\xd8") {
        Some("jpg")
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some("gif")
    } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        Some("webp")
    } else {
        None
    }
}

/// Convert an absolute filesystem path to a `file://` URL for the webview.
pub fn path_to_url(path: &Path) -> String {
    let url = url::Url::from_file_path(path).unwrap_or_else(|_| {
        // Fallback: return the path as-is; the webview may still load it on
        // some platforms, but this branch should rarely fire.
        url::Url::parse(&format!("file://{}", path.display()))
            .unwrap_or_else(|_| url::Url::parse("file:///").expect("file:/// is valid"))
    });
    url.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_normalizes_title() {
        assert_eq!(cache_key("Wild Life"), "wild-life");
        assert_eq!(cache_key("  Wild   LIFE  "), "wild-life");
    }

    #[test]
    fn image_ext_detects_png_jpg_gif_webp() {
        assert_eq!(image_ext(b"\x89PNG\r\n\x1a\nfoo"), Some("png"));
        assert_eq!(image_ext(b"\xff\xd8foo"), Some("jpg"));
        assert_eq!(image_ext(b"GIF89afoo"), Some("gif"));
        let webp = b"RIFFxxxxWEBP";
        assert_eq!(image_ext(webp), Some("webp"));
    }

    #[test]
    fn enrich_returns_baseline_without_external_calls() {
        let card = GameCard {
            slug: "test-unmatched-game-xyz".to_string(),
            title: "TestUnmatchedGameXYZ".to_string(),
            description: Some("A wild adventure".to_string()),
            developer: Some("Adeptus Steve".to_string()),
            external_genres: vec!["Adventure".to_string()],
            ..Default::default()
        };
        let (db, cfg) = (PathBuf::from(":memory:"), PathBuf::from("config.json"));
        let ctx = Context::new(db, cfg);
        let res = enrich(&ctx, &card).expect("enrich should succeed");
        assert_eq!(res.description.as_deref(), Some("A wild adventure"));
        assert_eq!(res.developer.as_deref(), Some("Adeptus Steve"));
        assert_eq!(res.genres, vec!["Adventure"]);
    }
}
