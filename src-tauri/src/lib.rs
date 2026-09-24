//! The Tauri app + native CLI share this crate (Rule 03: one core, two entry
//! points). Webview handlers are `#[tauri::command]`s over `core`; `main.rs`
//! routes argv to `cli_main` or the windowed `run`.
pub mod cli;

pub mod core;

pub mod db;
pub mod resolver;
pub mod scraper;

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::core::{Context, Error};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Managed runtime state handed to every GUI command (Rule 13: the webview
/// never touches the fs/SQLite itself — Rust owns both paths).
pub struct AppState {
    /// Resolved catalog + config paths (defaults unless `--db`/`--config`).
    pub context: Mutex<Context>,
    /// Non-blocking download queue + its background worker. `game_download`
    /// enqueues and returns immediately; the worker resolves + dispatches in the
    /// background so the webview never blocks on network (Rule 05) and users can
    /// schedule more downloads while any are active.
    pub queue: Arc<crate::core::queue::Queue>,
    /// In-memory cache for game profiles (slug -> Game).
    pub game_cache: Mutex<HashMap<String, crate::core::models::Game>>,
    /// In-memory cache for catalog pages (cache_key -> ArchivePage).
    pub page_cache: Mutex<HashMap<String, crate::scraper::archive::ArchivePage>>,
    /// In-memory cache for the genre cloud.
    pub genres_cache: Mutex<Option<Vec<crate::core::models::Genre>>>,
    /// In-memory cache for resolved artwork URLs ((key, kind) -> url).
    pub artwork_cache: Mutex<HashMap<(String, String), Option<String>>>,
}

/// Wire format returned to the Settings page (cluster of a single setting).
#[derive(Debug, Serialize, Deserialize)]
pub struct SettingView {
    pub key: String,
    pub value: serde_json::Value,
}

/// Fetch + parse one storefront archive page. The Store view calls this; the
/// CLI `sync`/`search` subcommands hit the same `core::catalog` function. The
/// optional `filter` mirrors the site's full GET filter form.
#[tauri::command]
fn catalog_page(
    state: tauri::State<'_, AppState>,
    page: Option<u32>,
    filter: Option<crate::core::models::ArchiveFilter>,
) -> Result<crate::scraper::archive::ArchivePage, String> {
    let p = page.unwrap_or(1);
    let filter = filter.unwrap_or_default();
    let cache_key = format!(
        "p:{}:{}:{:?}",
        p,
        filter.sort.as_deref().unwrap_or(""),
        filter
    );

    // 1. In-memory first
    if let Ok(cache) = state.page_cache.lock() {
        if let Some(archive) = cache.get(&cache_key) {
            return Ok(archive.clone());
        }
    }

    let ctx = state
        .context
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;

    let archive =
        crate::core::catalog::archive_page_filtered(&ctx, p, &filter).map_err(|e| e.to_string())?;

    // Persist cards to DB
    if let Ok(conn) = ctx.open_db() {
        for card in &archive.games {
            let post_id = card.post_id.unwrap_or_else(|| {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                card.slug.hash(&mut hasher);
                (hasher.finish() & 0x7FFF_FFFF_FFFF_FFFF) as i64
            });
            let _ = conn.execute(
                r#"
                INSERT INTO game (post_id, slug, title, thumbnail_url, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5)
                ON CONFLICT(slug) DO UPDATE SET
                    post_id = coalesce(game.post_id, excluded.post_id),
                    title = excluded.title,
                    thumbnail_url = coalesce(excluded.thumbnail_url, game.thumbnail_url),
                    updated_at = coalesce(excluded.updated_at, game.updated_at)
                "#,
                rusqlite::params![
                    post_id,
                    card.slug,
                    card.title,
                    card.thumb_url,
                    card.updated_at,
                ],
            );
        }
    }

    if let Ok(mut cache) = state.page_cache.lock() {
        cache.insert(cache_key, archive.clone());
    }

    Ok(archive)
}

/// The full `/game-genres/` tag cloud for the Store's left rail.
#[tauri::command]
fn catalog_genres(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<crate::core::models::Genre>, String> {
    // 1. In-memory first
    if let Ok(cache) = state.genres_cache.lock() {
        if let Some(genres) = &*cache {
            return Ok(genres.clone());
        }
    }

    let ctx = state
        .context
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;

    // 2. DB persistence check: if genre table has rows, load from DB
    if let Ok(conn) = ctx.open_db() {
        if let Ok(mut stmt) = conn.prepare("SELECT slug, label FROM genre ORDER BY label") {
            if let Ok(rows) = stmt.query_map([], |row| {
                Ok(crate::core::models::Genre {
                    slug: row.get(0)?,
                    label: row.get(1)?,
                    count: None,
                })
            }) {
                let genres: Vec<crate::core::models::Genre> = rows.filter_map(Result::ok).collect();
                if !genres.is_empty() {
                    if let Ok(mut cache) = state.genres_cache.lock() {
                        *cache = Some(genres.clone());
                    }
                    return Ok(genres);
                }
            }
        }
    }

    // 3. Fallback: fetch from network, persist to DB, and cache in memory
    let genres = crate::core::catalog::genres_page(&ctx).map_err(|e| e.to_string())?;

    if let Ok(conn) = ctx.open_db() {
        for g in &genres {
            let _ = conn.execute(
                "INSERT INTO genre (slug, label) VALUES (?1, ?2) ON CONFLICT(slug) DO UPDATE SET label = excluded.label",
                rusqlite::params![g.slug, g.label],
            );
        }
    }

    if let Ok(mut cache) = state.genres_cache.lock() {
        *cache = Some(genres.clone());
    }

    Ok(genres)
}

/// One genre's archive (`/game-genre/<slug>/page/N/?sort=`).
#[tauri::command]
fn catalog_genre(
    state: tauri::State<'_, AppState>,
    slug: String,
    page: Option<u32>,
    filter: Option<crate::core::models::ArchiveFilter>,
) -> Result<crate::scraper::archive::ArchivePage, String> {
    let p = page.unwrap_or(1);
    let sort = filter.as_ref().and_then(|f| f.sort.as_deref());
    let cache_key = format!("genre:{slug}:p:{p}:s:{}", sort.unwrap_or(""));

    // 1. In-memory first
    if let Ok(cache) = state.page_cache.lock() {
        if let Some(archive) = cache.get(&cache_key) {
            return Ok(archive.clone());
        }
    }

    let ctx = state
        .context
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;

    let archive =
        crate::core::catalog::genre_page(&ctx, &slug, p, sort).map_err(|e| e.to_string())?;

    if let Ok(mut cache) = state.page_cache.lock() {
        cache.insert(cache_key, archive.clone());
    }

    Ok(archive)
}

/// WordPress free-text search (`/?s=<query>`), same card shape as the archive.
#[tauri::command]
fn catalog_search(
    state: tauri::State<'_, AppState>,
    query: String,
) -> Result<crate::scraper::archive::ArchivePage, String> {
    let _ = state;
    if query.trim().is_empty() {
        return Err("a search query is required".to_string());
    }
    let url = crate::core::catalog::search_url(query.trim());
    let html = crate::scraper::fetch(&url).map_err(|e| e.to_string())?;
    Ok(crate::scraper::archive::parse_archive(&html))
}

/// One game's full page (`/game/<slug>/`) — powers the Store detail view.
#[tauri::command]
fn game_page(
    state: tauri::State<'_, AppState>,
    slug: String,
) -> Result<crate::core::models::Game, String> {
    let clean_slug = slug.trim_end_matches('/').to_string();

    // 1. In-memory first: return instantly if already loaded
    if let Ok(cache) = state.game_cache.lock() {
        if let Some(game) = cache.get(&clean_slug) {
            return Ok(game.clone());
        }
    }

    let ctx = state
        .context
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;

    // 2. DB persistence: if present in SQLite DB with versions or entries, load into memory and return
    if let Ok(conn) = ctx.open_db() {
        if let Ok(Some(mut game)) = crate::db::repo::game_by_slug(&conn, &clean_slug) {
            if !game.versions.is_empty() || !game.download_entries.is_empty() {
                // If game in DB has fewer than 2 screenshots or missing description, enrich it from external providers
                if game.screenshots.len() < 2 || game.description.is_none() {
                    let card = crate::core::models::GameCard {
                        slug: game.slug.clone(),
                        title: game.title.clone(),
                        post_id: game.post_id,
                        thumb_url: game.screenshots.first().cloned(),
                        description: game.description.clone(),
                        developer: game.developer.clone(),
                        genres: game.genres.clone(),
                        ..Default::default()
                    };
                    if let Ok(extra) = crate::core::content::enrich(&ctx, &card) {
                        if (game.description.is_none()
                            || game.description.as_deref().unwrap_or("").trim().is_empty())
                            && extra.description.is_some()
                        {
                            game.description = extra.description;
                        }
                        if (game.developer.is_none()
                            || game.developer.as_deref().unwrap_or("").trim().is_empty())
                            && extra.developer.is_some()
                        {
                            game.developer = extra.developer;
                        }
                        for g in extra.genres {
                            if !game.genres.contains(&g) {
                                game.genres.push(g);
                            }
                        }
                        for s in extra.screenshots {
                            if !game.screenshots.contains(&s) && game.screenshots.len() < 10 {
                                game.screenshots.push(s);
                            }
                        }
                        let _ = crate::db::repo::upsert_game(&conn, &game, None, None);
                    }
                }
                if let Ok(mut cache) = state.game_cache.lock() {
                    cache.insert(clean_slug.clone(), game.clone());
                }
                return Ok(game);
            }
        }
    }

    // 3. Fallback: fetch from network, persist to DB, and cache in memory
    let url = format!("https://lewdzone.com/game/{clean_slug}/");
    let html = crate::scraper::fetch(&url).map_err(|e| e.to_string())?;
    let mut game = crate::scraper::game::parse_game(&html);
    if game.slug.is_empty() {
        game.slug = clean_slug.clone();
    }

    let card = crate::core::models::GameCard {
        slug: game.slug.clone(),
        title: game.title.clone(),
        post_id: game.post_id,
        thumb_url: game.screenshots.first().cloned(),
        description: game.description.clone(),
        developer: game.developer.clone(),
        genres: game.genres.clone(),
        ..Default::default()
    };
    if let Ok(extra) = crate::core::content::enrich(&ctx, &card) {
        if game.description.is_none() || game.description.as_deref().unwrap_or("").trim().is_empty()
        {
            game.description = extra.description;
        }
        if game.developer.is_none() || game.developer.as_deref().unwrap_or("").trim().is_empty() {
            game.developer = extra.developer;
        }
        for g in extra.genres {
            if !game.genres.contains(&g) {
                game.genres.push(g);
            }
        }
        for s in extra.screenshots {
            if !game.screenshots.contains(&s) && game.screenshots.len() < 10 {
                game.screenshots.push(s);
            }
        }
    }

    let cover_thumbnail = crate::scraper::game::parse_cover_url(&html);

    // Persist to DB
    if let Ok(conn) = ctx.open_db() {
        let _ = crate::db::repo::upsert_game(&conn, &game, None, cover_thumbnail.as_deref());
    }

    // Store in memory cache
    if let Ok(mut cache) = state.game_cache.lock() {
        cache.insert(clean_slug, game.clone());
    }

    Ok(game)
}

/// The sources available for a game's Download panel, ordered by the user's
/// `source-priority` with the preferred host flagged as the dropdown default.
#[tauri::command]
fn game_sources(
    state: tauri::State<'_, AppState>,
    slug: String,
    version: Option<String>,
    platform: Option<String>,
    tab: Option<String>,
) -> Result<Vec<crate::core::download::HostSource>, String> {
    let ctx = state
        .context
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    let clean_slug = slug.trim_end_matches('/');

    // 1. In-memory first
    let cached_game = state
        .game_cache
        .lock()
        .ok()
        .and_then(|c| c.get(clean_slug).cloned());

    // 2. DB persistence check
    let game = match cached_game {
        Some(g) => Some(g),
        None => {
            if let Ok(conn) = ctx.open_db() {
                if let Ok(Some(g)) = crate::db::repo::game_by_slug(&conn, clean_slug) {
                    if let Ok(mut cache) = state.game_cache.lock() {
                        cache.insert(clean_slug.to_string(), g.clone());
                    }
                    Some(g)
                } else {
                    None
                }
            } else {
                None
            }
        }
    };

    if let Some(game) = game {
        let settings =
            crate::core::settings::Settings::load(&ctx.config_path).map_err(|e| e.to_string())?;
        return crate::core::download::sources_for(
            &game,
            version.as_deref().unwrap_or("latest"),
            platform.as_deref().unwrap_or("PC"),
            tab.as_deref().unwrap_or("official"),
            settings.source_priority.as_deref(),
        )
        .map_err(|e| e.to_string());
    }

    let select = crate::core::download::Select {
        game: clean_slug,
        version: version.as_deref().unwrap_or("latest"),
        platform: platform.as_deref().unwrap_or("PC"),
        tab: tab.as_deref().unwrap_or("official"),
        source: None,
    };
    let jobs = crate::core::download::sources_for_selection(&ctx, &select, &mut |url| {
        crate::scraper::fetch(url)
    })
    .map_err(|e| e.to_string())?;
    Ok(jobs)
}

/// Enqueue a game's download for the background worker — the Store's "Download"
/// button. Returns instantly with the queued job row; the worker (owned Context,
/// never the state lock) resolves go-links and dispatches (in-app stream or
/// OS handler) off the webview thread, so the UI stays interactive and more
/// downloads can be scheduled while any are active (Rule 05). `source` (chosen from the game page
/// dropdown) restricts to one host; `None` = all available sources.
#[tauri::command]
fn game_download(
    state: tauri::State<'_, AppState>,
    slug: String,
    version: Option<String>,
    platform: Option<String>,
    tab: Option<String>,
    source: Option<String>,
) -> Result<crate::core::queue::QueueJob, String> {
    state
        .queue
        .enqueue(
            slug.trim_end_matches('/').to_string(),
            version.unwrap_or_else(|| "latest".to_string()),
            platform.unwrap_or_else(|| "PC".to_string()),
            tab.unwrap_or_else(|| "official".to_string()),
            source,
        )
        .map_err(|e| e.to_string())
}

/// Snapshot of the download queue for the Downloads view (newest first).
#[tauri::command]
fn downloads_list(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<crate::core::queue::QueueJob>, String> {
    Ok(state.queue.snapshot())
}

/// Favorites list (heart-tab). Returns `Vec<GameCard>` newest-first.
#[tauri::command]
fn favorites_list(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<crate::core::models::GameCard>, String> {
    let ctx = state
        .context
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    crate::core::favorites::list(&ctx).map_err(|e| e.to_string())
}

/// Add a game to favorites by slug.
#[tauri::command]
fn favorite_add(state: tauri::State<'_, AppState>, slug: String) -> Result<(), String> {
    let ctx = state
        .context
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    crate::core::favorites::add(&ctx, &slug).map_err(|e| e.to_string())
}

/// Remove a game from favorites by slug.
#[tauri::command]
fn favorite_remove(state: tauri::State<'_, AppState>, slug: String) -> Result<(), String> {
    let ctx = state
        .context
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    crate::core::favorites::remove(&ctx, &slug).map_err(|e| e.to_string())
}

/// Create a native desktop shortcut for an installed game.
#[tauri::command]
fn create_shortcut(state: tauri::State<'_, AppState>, slug: String) -> Result<String, String> {
    let ctx = state
        .context
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    let paths = crate::core::shortcuts::create(
        &ctx,
        &slug,
        crate::core::shortcuts::ShortcutOptions::default(),
    )
    .map_err(|e| e.to_string())?;
    let summary = paths
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    Ok(summary)
}

/// Installed games from the library manifests (ADR-0005).
#[tauri::command]
fn library_list(
    state: tauri::State<'_, AppState>,
) -> Result<crate::core::list::LibraryListing, String> {
    let ctx = state
        .context
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    crate::core::list::library_listing(&ctx).map_err(|e| e.to_string())
}

/// Scan a directory for extracted games and register them in the library.
#[tauri::command]
fn library_scan(
    state: tauri::State<'_, AppState>,
    path: Option<String>,
) -> Result<crate::core::library::ScanReport, String> {
    let ctx = state
        .context
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    crate::core::library::scan_games_dir(
        &ctx,
        path.as_deref().map(std::path::Path::new),
        Some(&state.queue),
    )
    .map_err(|e| e.to_string())
}

/// Launch an installed game from its `lzapps/<slug>/app.json` manifest.
#[tauri::command]
fn game_launch(state: tauri::State<'_, AppState>, slug: String) -> Result<(), String> {
    let ctx = state
        .context
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    crate::core::launch::run(&ctx, &slug).map_err(|e| e.to_string())?;
    Ok(())
}

/// Open an installed game's directory in the native file manager.
#[tauri::command]
fn open_game_folder(state: tauri::State<'_, AppState>, slug: String) -> Result<(), String> {
    let ctx = state
        .context
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    let listing = crate::core::list::library_listing(&ctx).map_err(|e| e.to_string())?;
    if let Some(game) = listing.games.into_iter().find(|g| g.slug == slug) {
        let path = std::path::PathBuf::from(&game.install_path);
        if path.exists() {
            #[cfg(target_os = "windows")]
            {
                std::process::Command::new("explorer")
                    .arg(&path)
                    .spawn()
                    .map_err(|e| e.to_string())?;
                return Ok(());
            }
            #[cfg(target_os = "macos")]
            {
                std::process::Command::new("open")
                    .arg(&path)
                    .spawn()
                    .map_err(|e| e.to_string())?;
                return Ok(());
            }
            #[cfg(all(unix, not(target_os = "macos")))]
            {
                std::process::Command::new("xdg-open")
                    .arg(&path)
                    .spawn()
                    .map_err(|e| e.to_string())?;
                return Ok(());
            }
        }
    }
    Err("Game install folder not found".to_string())
}

/// Uninstall an installed game by deleting its install directory and manifest.
#[tauri::command]
fn uninstall_game(state: tauri::State<'_, AppState>, slug: String) -> Result<(), String> {
    let ctx = state
        .context
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    let listing = crate::core::list::library_listing(&ctx).map_err(|e| e.to_string())?;
    if let Some(game) = listing.games.into_iter().find(|g| g.slug == slug) {
        let path = std::path::PathBuf::from(&game.install_path);
        if path.exists() {
            let _ = std::fs::remove_dir_all(&path);
        }
        if let Ok(lz_root) = crate::core::folder::lzapps_root(&ctx) {
            let manifest_dir = lz_root.join(&slug);
            if manifest_dir.exists() {
                let _ = std::fs::remove_dir_all(&manifest_dir);
            }
        }
        return Ok(());
    }
    Err("Game not found in library".to_string())
}

/// Best-effort enrichment for a game card from external content providers.
#[tauri::command]
fn content_enrich(
    state: tauri::State<'_, AppState>,
    card: crate::core::models::GameCard,
) -> Result<crate::core::content::Enrichment, String> {
    let ctx = state
        .context
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    crate::core::content::enrich(&ctx, &card).map_err(|e| e.to_string())
}

/// Return an artwork URL for a game card. LewdZone's own thumbnail is the
/// default cover source; external providers are only queried when it is
/// missing or when a non-cover kind is requested.
#[tauri::command]
fn artwork_url(
    state: tauri::State<'_, AppState>,
    card: crate::core::models::GameCard,
    kind: String,
) -> Result<Option<String>, String> {
    let cache_key = (
        if !card.slug.is_empty() {
            card.slug.clone()
        } else {
            card.post_id.map(|p| p.to_string()).unwrap_or_default()
        },
        kind.clone(),
    );

    // 1. In-memory first
    if let Ok(cache) = state.artwork_cache.lock() {
        if let Some(cached) = cache.get(&cache_key) {
            return Ok(cached.clone());
        }
    }

    let parsed_kind = match kind.as_str() {
        "icon" => crate::core::content::ArtworkKind::Icon,
        "cover" => crate::core::content::ArtworkKind::Cover,
        "background" => crate::core::content::ArtworkKind::Background,
        _ => return Err(format!("unknown artwork kind: {kind}")),
    };

    let result = (|| -> Result<Option<String>, String> {
        let ctx = state
            .context
            .lock()
            .map_err(|_| "state lock poisoned".to_string())?;

        // LewdZone is the authoritative source for its own cover thumbnail.
        if parsed_kind == crate::core::content::ArtworkKind::Cover {
            if let Some(ref t) = card.thumb_url {
                if let Some(clean) = crate::db::repo::clean_thumbnail(t) {
                    return Ok(Some(clean));
                }
            }
            // Fall back to SQLite database lookup by slug or post_id
            if let Ok(conn) = crate::db::open(&ctx.db_path) {
                let db_thumb = if !card.slug.is_empty() {
                    crate::db::repo::thumbnail_by_slug(&conn, &card.slug)
                        .ok()
                        .flatten()
                } else if let Some(pid) = card.post_id {
                    crate::db::repo::thumbnail_by_post_id(&conn, pid)
                        .ok()
                        .flatten()
                } else {
                    None
                };
                if let Some(ref t) = db_thumb {
                    if let Some(clean) = crate::db::repo::clean_thumbnail(t) {
                        return Ok(Some(clean));
                    }
                }
            }

            // Fall back to resolving from LewdZone directly (for already installed games or games not yet in SQLite)
            if !card.slug.is_empty() {
                if let Some(lz_thumb) = resolve_lewdzone_thumbnail(&ctx, &card.slug, &card.title) {
                    return Ok(Some(lz_thumb));
                }
            }
        }

        crate::core::content::artwork(&ctx, &card, parsed_kind)
            .map(|opt| opt.map(|p| crate::core::content::path_to_url(&p)))
            .map_err(|e| e.to_string())
    })();

    if let Ok(ref val) = result {
        if let Ok(mut cache) = state.artwork_cache.lock() {
            cache.insert(cache_key, val.clone());
        }
    }

    result
}

fn resolve_lewdzone_thumbnail(
    ctx: &crate::core::Context,
    slug: &str,
    title: &str,
) -> Option<String> {
    // 1. Direct game page fetch: https://lewdzone.com/game/{slug}/
    let url = format!("https://lewdzone.com/game/{slug}/");
    if let Ok(html) = crate::scraper::fetch(&url) {
        if let Some(thumb) = crate::scraper::game::parse_cover_url(&html) {
            if let Ok(conn) = ctx.open_db() {
                let post_id = {
                    use std::hash::{Hash, Hasher};
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    slug.hash(&mut hasher);
                    (hasher.finish() & 0x7FFF_FFFF_FFFF_FFFF) as i64
                };
                let _ = conn.execute(
                    r#"
                    INSERT INTO game (post_id, slug, title, thumbnail_url)
                    VALUES (?1, ?2, ?3, ?4)
                    ON CONFLICT(slug) DO UPDATE SET
                        thumbnail_url = coalesce(excluded.thumbnail_url, game.thumbnail_url)
                    "#,
                    rusqlite::params![post_id, slug, title, thumb],
                );
            }
            return Some(thumb);
        }
    }

    // 2. Search on lewdzone.com by title (or slug)
    let q = if !title.trim().is_empty() {
        title.trim()
    } else {
        slug.trim()
    };
    let search_url = crate::core::catalog::search_url(q);
    if let Ok(html) = crate::scraper::fetch(&search_url) {
        let archive = crate::scraper::archive::parse_archive(&html);
        let found = archive
            .games
            .iter()
            .find(|g| g.slug == slug)
            .or_else(|| archive.games.first());
        if let Some(c) = found {
            if let Some(ref thumb) = c.thumb_url {
                if let Ok(conn) = ctx.open_db() {
                    let post_id = c.post_id.unwrap_or_else(|| {
                        use std::hash::{Hash, Hasher};
                        let mut hasher = std::collections::hash_map::DefaultHasher::new();
                        c.slug.hash(&mut hasher);
                        (hasher.finish() & 0x7FFF_FFFF_FFFF_FFFF) as i64
                    });
                    let _ = conn.execute(
                        r#"
                        INSERT INTO game (post_id, slug, title, thumbnail_url)
                        VALUES (?1, ?2, ?3, ?4)
                        ON CONFLICT(slug) DO UPDATE SET
                            thumbnail_url = coalesce(excluded.thumbnail_url, game.thumbnail_url)
                        "#,
                        rusqlite::params![
                            post_id,
                            &c.slug,
                            if title.is_empty() { &c.title } else { title },
                            thumb
                        ],
                    );
                }
                return Some(thumb.clone());
            }
        }
    }

    None
}

/// Cancel a queued or active download job.
#[tauri::command]
fn download_cancel(state: tauri::State<'_, AppState>, id: u64) -> Result<(), String> {
    state.queue.cancel(id).map_err(|e| e.to_string())
}

/// Delete a finished or failed download job from queue.
#[tauri::command]
fn download_delete(state: tauri::State<'_, AppState>, id: u64) -> Result<(), String> {
    state.queue.delete(id).map_err(|e| e.to_string())
}

/// Clear all completed or failed jobs from the queue.
#[tauri::command]
fn downloads_clear(state: tauri::State<'_, AppState>) -> Result<usize, String> {
    state.queue.clear_finished().map_err(|e| e.to_string())
}

/// Resolve a go-link using the backend resolver.
#[tauri::command]
fn resolve_go_link(go_link: String) -> Result<crate::resolver::ResolvedUrl, String> {
    crate::resolver::resolve(&go_link).map_err(|e| e.to_string())
}

/// Fully terminate the application (called from the GUI File > Quit action).
#[tauri::command]
fn app_quit(app: tauri::AppHandle) {
    app.exit(0);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedDownloadPayload {
    pub slug: String,
    pub url: String,
}

/// Open a child webview window running our own custom in-app redirect page,
/// ensuring no third-party malicious ads, popups, or tracking scripts run.
#[tauri::command]
async fn open_resolver_window(
    app: tauri::AppHandle,
    slug: String,
    url: String,
    host: Option<String>,
    title: Option<String>,
) -> Result<(), String> {
    use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

    if let Some(win) = app.get_webview_window("download-resolver") {
        let _ = win.close();
    }

    let enc_url: String = url::form_urlencoded::byte_serialize(url.as_bytes()).collect();
    let enc_slug: String = url::form_urlencoded::byte_serialize(slug.as_bytes()).collect();
    let host_val = host.as_deref().unwrap_or("");
    let enc_host: String = url::form_urlencoded::byte_serialize(host_val.as_bytes()).collect();
    let title_val = title.as_deref().unwrap_or(&slug);
    let enc_title: String = url::form_urlencoded::byte_serialize(title_val.as_bytes()).collect();

    let path = format!("resolver?slug={enc_slug}&url={enc_url}&host={enc_host}&title={enc_title}");

    let builder =
        WebviewWindowBuilder::new(&app, "download-resolver", WebviewUrl::App(path.into()))
            .title(format!(
                "Download Verification - {}",
                title.as_deref().unwrap_or(&slug)
            ))
            .inner_size(700.0, 560.0)
            .min_inner_size(520.0, 420.0)
            .center();

    builder.build().map_err(|e| e.to_string())?;
    Ok(())
}

/// `settings.get()` — one value or the whole snapshot as JSON for the page.
#[tauri::command]
fn settings_get(
    state: tauri::State<'_, AppState>,
    key: Option<String>,
) -> Result<serde_json::Value, String> {
    let ctx = state
        .context
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    match key {
        Some(k) => settings_value(&ctx, &k)
            .map(|v| v.unwrap_or(serde_json::Value::Null))
            .map_err(|e| e.to_string()),
        None => crate::core::settings::load_snapshot(&ctx).map_err(|e| e.to_string()),
    }
}

/// `settings.set(key, value, secret?)` — persist and return the new value.
/// Secret keys (or `secret: true`) are stored in the SQLite `secret` table and
/// the returned value is a presence marker, never the raw secret.
#[tauri::command]
fn settings_set(
    state: tauri::State<'_, AppState>,
    key: String,
    value: String,
    secret: Option<bool>,
) -> Result<SettingView, String> {
    let ctx = state
        .context
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    let value = crate::core::settings::apply(&ctx, &key, &value, secret.unwrap_or(false))
        .map_err(|e| e.to_string())?;
    Ok(SettingView { key, value })
}

/// Installed theme skins (sorted names; the default theme is implicit).
#[tauri::command]
fn themes_list(state: tauri::State<'_, AppState>) -> Result<Vec<String>, String> {
    let _ = state;
    crate::core::skins::installed().map_err(|e| e.to_string())
}

/// Effective tokens for the active theme — the frontend swaps these CSS
/// custom props on `:root` at runtime (no restart; ADR-0005).
#[tauri::command]
fn themes_tokens(state: tauri::State<'_, AppState>) -> Result<Vec<(String, String)>, String> {
    let ctx = state
        .context
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    let tokens = tokens_for(&ctx)?;
    Ok(tokens.into_iter().collect())
}

/// Select a skin (name from `themes_list`, or empty/absent = default) and
/// return the tokens for exactly that selection so the UI applies the theme
/// the user picked — regardless of what was active before.
#[tauri::command]
fn themes_apply(
    state: tauri::State<'_, AppState>,
    name: Option<String>,
) -> Result<Vec<(String, String)>, String> {
    let ctx = state
        .context
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    let effective = match name.as_deref() {
        Some("") | None => "",
        Some(n) => n,
    };
    crate::core::settings::apply(&ctx, "theme", effective, false)?;
    let tokens = crate::core::skins::resolve(Some(effective))?;
    Ok(tokens.into_iter().collect())
}

fn settings_value(ctx: &Context, key: &str) -> Result<Option<serde_json::Value>, Error> {
    let s = crate::core::settings::Settings::load(&ctx.config_path)?;
    Ok(s.get_value(key))
}

fn tokens_for(ctx: &Context) -> Result<BTreeMap<String, String>, Error> {
    let s = crate::core::settings::Settings::load(&ctx.config_path)?;
    crate::core::skins::resolve(s.theme.as_deref())
}

/// Get the full path to the debug log file.
#[tauri::command]
fn get_debug_log_path() -> Option<String> {
    crate::core::paths::log_file_path().map(|p| p.to_string_lossy().to_string())
}

/// Open the platform-standard logs folder in the OS file manager.
#[tauri::command]
fn open_debug_log_folder() -> Result<(), String> {
    if let Some(dir) = crate::core::paths::logs_dir() {
        let _ = std::fs::create_dir_all(&dir);
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("explorer")
                .arg(&dir)
                .spawn()
                .map_err(|e| e.to_string())?;
            return Ok(());
        }
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(&dir)
                .spawn()
                .map_err(|e| e.to_string())?;
            return Ok(());
        }
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            std::process::Command::new("xdg-open")
                .arg(&dir)
                .spawn()
                .map_err(|e| e.to_string())?;
            return Ok(());
        }
    }
    Err("Logs directory could not be determined".to_string())
}

/// Clear the debug log file.
#[tauri::command]
fn clear_debug_log() -> Result<(), String> {
    crate::core::logging::clear_log().map_err(|e| e.to_string())
}

/// Read recent lines from the debug log.
#[tauri::command]
fn read_debug_log() -> Result<String, String> {
    crate::core::logging::read_log_tail(200).map_err(|e| e.to_string())
}

/// Get installer detection status (is installed, version, default path, etc.).
#[tauri::command]
fn installer_status() -> crate::core::installer::InstallerStatus {
    crate::core::installer::detect_status()
}

/// Check available disk space on the given path.
#[tauri::command]
fn installer_disk_space(target_dir: String) -> crate::core::installer::DiskSpaceInfo {
    crate::core::installer::check_disk_space(&target_dir)
}

/// Execute application installation.
#[tauri::command]
fn installer_install(
    options: crate::core::installer::InstallOptions,
) -> crate::core::installer::OperationResult {
    crate::core::installer::perform_install(options)
}

/// Execute application uninstallation.
#[tauri::command]
fn installer_uninstall(
    options: crate::core::installer::UninstallOptions,
) -> crate::core::installer::OperationResult {
    crate::core::installer::perform_uninstall(options)
}

/// Open the installer/uninstaller window.
#[tauri::command]
async fn open_installer_window(app: tauri::AppHandle, mode: Option<String>) -> Result<(), String> {
    use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

    if let Some(win) = app.get_webview_window("installer") {
        let _ = win.show();
        let _ = win.set_focus();
        return Ok(());
    }

    let mode_str = mode.unwrap_or_else(|| "auto".to_string());
    let path = format!("installer?mode={mode_str}");

    let builder = WebviewWindowBuilder::new(&app, "installer", WebviewUrl::App(path.into()))
        .title("LewdZone Launcher Setup")
        .inner_size(780.0, 560.0)
        .min_inner_size(720.0, 520.0)
        .resizable(false)
        .center();

    builder.build().map_err(|e| e.to_string())?;
    Ok(())
}

/// CLI entry point called from `main.rs` when argv has subcommands.
pub fn cli_main() -> std::process::ExitCode {
    use clap::Parser;

    let (_db, config) = default_context();
    let initial_debug = crate::core::settings::Settings::load(&config)
        .ok()
        .and_then(|s| s.debug_logging)
        .unwrap_or(false);
    crate::core::logging::init(crate::core::paths::log_file_path(), initial_debug);

    let cli = cli::Cli::parse();
    let code = cli::run(cli);
    std::process::ExitCode::from(code.as_i32() as u8)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (db, config) = default_context();
    let initial_debug = crate::core::settings::Settings::load(&config)
        .ok()
        .and_then(|s| s.debug_logging)
        .unwrap_or(false);
    crate::core::logging::init(crate::core::paths::log_file_path(), initial_debug);
    let ctx = Context::new(db.clone(), config.clone());
    let queue = Arc::new(
        crate::core::queue::Queue::load(&ctx).expect("queue should load from the local database"),
    );
    // Background download worker: owns its Context clone so it never touches the
    // state lock; the webview enqueues via `game_download` and stays responsive.
    let _worker = crate::core::queue::spawn_worker(queue.clone(), ctx);
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            context: Mutex::new(Context::new(db, config)),
            queue,
            game_cache: Mutex::new(HashMap::new()),
            page_cache: Mutex::new(HashMap::new()),
            genres_cache: Mutex::new(None),
            artwork_cache: Mutex::new(HashMap::new()),
        })
        .setup(|app| {
            use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
            use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
            use tauri::Manager;

            let show_i = MenuItem::with_id(app, "show", "Show LewdZone", true, None::<&str>)?;
            let min_i = MenuItem::with_id(app, "minimize", "Minimize to Tray", true, None::<&str>)?;
            let sep = PredefinedMenuItem::separator(app)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&show_i, &min_i, &sep, &quit_i])?;

            let mut tray_builder = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false)
                .tooltip("LewdZone Launcher")
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "minimize" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                    }
                });

            let tray_icon =
                tauri::image::Image::from_bytes(include_bytes!("../icons/tray-icon.png")).ok();
            if let Some(icon) = tray_icon.or_else(|| app.default_window_icon().cloned()) {
                tray_builder = tray_builder.icon(icon);
            }

            tray_builder.build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            settings_get,
            settings_set,
            themes_list,
            themes_tokens,
            themes_apply,
            catalog_page,
            catalog_genres,
            catalog_genre,
            catalog_search,
            game_page,
            game_sources,
            game_download,
            downloads_list,
            favorites_list,
            favorite_add,
            favorite_remove,
            library_list,
            library_scan,
            create_shortcut,
            game_launch,
            content_enrich,
            artwork_url,
            open_resolver_window,
            download_cancel,
            download_delete,
            downloads_clear,
            resolve_go_link,
            app_quit,
            get_debug_log_path,
            open_debug_log_folder,
            clear_debug_log,
            read_debug_log,
            installer_status,
            installer_disk_space,
            installer_install,
            installer_uninstall,
            open_installer_window,
            open_game_folder,
            uninstall_game
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Launch the application directly into the unified installer/uninstaller wizard window.
pub fn run_installer() {
    let (db, config) = default_context();
    let initial_debug = crate::core::settings::Settings::load(&config)
        .ok()
        .and_then(|s| s.debug_logging)
        .unwrap_or(false);
    crate::core::logging::init(crate::core::paths::log_file_path(), initial_debug);
    let ctx = Context::new(db.clone(), config.clone());
    let queue = Arc::new(
        crate::core::queue::Queue::load(&ctx).expect("queue should load from the local database"),
    );

    let is_uninstall = std::env::args().any(|a| a == "--uninstall" || a == "--maintenance");
    let mode = if is_uninstall { "uninstall" } else { "install" };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            context: Mutex::new(Context::new(db, config)),
            queue,
            game_cache: Mutex::new(HashMap::new()),
            page_cache: Mutex::new(HashMap::new()),
            genres_cache: Mutex::new(None),
            artwork_cache: Mutex::new(HashMap::new()),
        })
        .setup(move |app| {
            use tauri::{WebviewUrl, WebviewWindowBuilder};

            let path = format!("installer?mode={mode}");
            let _ = WebviewWindowBuilder::new(app, "installer", WebviewUrl::App(path.into()))
                .title("LewdZone Launcher Setup")
                .inner_size(780.0, 560.0)
                .min_inner_size(720.0, 520.0)
                .resizable(false)
                .center()
                .build()?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            settings_get,
            settings_set,
            themes_list,
            themes_tokens,
            themes_apply,
            catalog_page,
            catalog_genres,
            catalog_genre,
            catalog_search,
            game_page,
            game_sources,
            game_download,
            downloads_list,
            favorites_list,
            favorite_add,
            favorite_remove,
            library_list,
            library_scan,
            create_shortcut,
            game_launch,
            content_enrich,
            artwork_url,
            open_resolver_window,
            download_cancel,
            download_delete,
            downloads_clear,
            resolve_go_link,
            app_quit,
            get_debug_log_path,
            open_debug_log_folder,
            clear_debug_log,
            read_debug_log,
            installer_status,
            installer_disk_space,
            installer_install,
            installer_uninstall,
            open_installer_window,
            open_game_folder,
            uninstall_game
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri installer application");
}

fn default_context() -> (PathBuf, PathBuf) {
    let db = crate::core::paths::default_db_path().unwrap_or_else(|| PathBuf::from("lewdzone.db"));
    let config =
        crate::core::paths::default_config_path().unwrap_or_else(|| PathBuf::from("config.json"));
    (db, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_context_is_a_real_pair() {
        let (db, config) = default_context();
        assert!(db.file_name().is_some());
        assert!(config.file_name().is_some());
        assert_ne!(db, config);
    }
}
