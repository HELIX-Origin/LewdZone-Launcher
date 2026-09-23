//! Content-provider layer: enrich games and cache artwork from external
//! sources (SteamGridDB, IGDB, VNDB, itch.io, Steam, IndieDB).
//!
//! LewdZone remains authoritative for downloads; providers only fill missing
//! presentation fields and are never allowed to overwrite scraped values.
//!
//! All API keys are read from the SQLite `secret` table (Rule 10). Providers
//! that need a key and do not have one are silently skipped.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::models::GameCard;
use crate::core::{paths, Context, Error};
use crate::db;

pub mod igdb;
pub mod indiedb;
pub mod itch;
pub mod steam;
pub mod steamgriddb;
pub mod vndb;

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

/// Enrichment data returned by a provider for a single game.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Enrichment {
    pub description: Option<String>,
    pub developer: Option<String>,
    pub rating: Option<f32>,
    pub tags: Vec<String>,
    pub screenshots: Vec<String>,
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
    ) -> Result<Option<Vec<u8>>, Error>;
}

/// All registered providers, in priority order.
pub fn providers() -> Vec<Box<dyn Provider>> {
    vec![
        Box::new(steamgriddb::SteamGridDb),
        Box::new(vndb::Vndb),
        Box::new(igdb::Igdb),
        Box::new(itch::Itch),
        Box::new(steam::Steam),
        Box::new(indiedb::IndieDb),
    ]
}

/// Stable cache key for a game title. Lowercase, collapsed whitespace.
pub fn cache_key(title: &str) -> String {
    title
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

/// Load all secrets this layer cares about.
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

/// Return the directory where artwork files are cached.
pub fn artwork_dir() -> Result<PathBuf, Error> {
    paths::library_artwork_dir().ok_or_else(|| {
        Error::Runtime("cannot resolve artwork cache directory (no app data dir)".to_string())
    })
}

/// Best-effort enrichment for a game. Each enabled provider is tried; the
/// first result wins per field. Records are stored in `game_external`.
pub fn enrich(ctx: &Context, card: &GameCard) -> Result<Enrichment, Error> {
    let secrets = load_secrets(ctx)?;
    let mut merged = Enrichment::default();
    let mut stored_any = false;

    for provider in providers() {
        if !provider.enabled(&secrets) {
            continue;
        }
        match provider.enrich(&secrets, card) {
            Ok(Some(mut data)) => {
                // Serialize the raw provider payload before we take fields.
                let raw_json = serde_json::to_string(&data).unwrap_or_default();
                let external_id = data.external_id_or(card);

                if merged.description.is_none() {
                    merged.description = data.description.take();
                }
                if merged.developer.is_none() {
                    merged.developer = data.developer.take();
                }
                if merged.rating.is_none() {
                    merged.rating = data.rating.take();
                }
                if merged.tags.is_empty() && !data.tags.is_empty() {
                    merged.tags = data.tags;
                }
                if merged.screenshots.is_empty() && !data.screenshots.is_empty() {
                    merged.screenshots = data.screenshots;
                }
                if let Some(post_id) = card.post_id {
                    let conn = db::open(&ctx.db_path)?;
                    db::repo::game_external_upsert(
                        &conn,
                        post_id,
                        provider.name(),
                        &external_id,
                        &raw_json,
                    )?;
                    stored_any = true;
                }
            }
            Ok(None) => {}
            Err(e) => {
                // Providers are best-effort; log and continue.
                eprintln!(
                    "[content] {} enrich failed for {}: {}",
                    provider.name(),
                    card.slug,
                    e
                );
            }
        }
    }

    if stored_any {
        if let Some(post_id) = card.post_id {
            let _ = db::repo::game_external_list(&db::open(&ctx.db_path)?, post_id);
        }
    }

    Ok(merged)
}

impl Enrichment {
    fn external_id_or(&self, card: &GameCard) -> String {
        // SteamGridDB and others use the title as the lookup key when no
        // explicit external id is returned.
        card.slug.clone()
    }
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

    // Check the database cache first.
    {
        let conn = db::open(&ctx.db_path)?;
        if let Some(row) = db::repo::artwork_cache_get(&conn, &key, &kind.to_string())? {
            let path = PathBuf::from(&row.file_path);
            if path.exists() {
                return Ok(Some(path));
            }
        }
    }

    let secrets = load_secrets(ctx)?;
    for provider in providers() {
        if !provider.enabled(&secrets) {
            continue;
        }
        match provider.artwork(&secrets, card, kind) {
            Ok(Some(bytes)) if !bytes.is_empty() => {
                let ext = image_ext(&bytes).unwrap_or("jpg");
                let file_name = format!("{}-{}.{ext}", key, kind);
                let path = dir.join(&file_name);
                let mut file = fs::File::create(&path)?;
                file.write_all(&bytes)?;

                let conn = db::open(&ctx.db_path)?;
                db::repo::artwork_cache_upsert(
                    &conn,
                    &key,
                    &kind.to_string(),
                    provider.name(),
                    &path.to_string_lossy(),
                )?;
                return Ok(Some(path));
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!(
                    "[content] {} artwork failed for {} {:?}: {}",
                    provider.name(),
                    card.slug,
                    kind,
                    e
                );
            }
        }
    }

    Ok(None)
}

/// Guess an image extension from the first bytes.
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
}
