//! The Tauri app + native CLI share this crate (Rule 03: one core, two entry
//! points). Webview handlers are `#[tauri::command]`s over `core`; `main.rs`
//! routes argv to `cli_main` or the windowed `run`.
pub mod cli;

pub mod core;

pub mod db;
pub mod dm;
pub mod resolver;
pub mod scraper;

use std::collections::BTreeMap;
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
    let ctx = state
        .context
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    let filter = filter.unwrap_or_default();
    crate::core::catalog::archive_page_filtered(&ctx, page.unwrap_or(1), &filter)
        .map_err(|e| e.to_string())
}

/// The full `/game-genres/` tag cloud for the Store's left rail.
#[tauri::command]
fn catalog_genres(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<crate::core::models::Genre>, String> {
    let ctx = state
        .context
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    crate::core::catalog::genres_page(&ctx).map_err(|e| e.to_string())
}

/// One genre's archive (`/game-genre/<slug>/page/N/?sort=`).
#[tauri::command]
fn catalog_genre(
    state: tauri::State<'_, AppState>,
    slug: String,
    page: Option<u32>,
    filter: Option<crate::core::models::ArchiveFilter>,
) -> Result<crate::scraper::archive::ArchivePage, String> {
    let ctx = state
        .context
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    let sort = filter.and_then(|f| f.sort);
    crate::core::catalog::genre_page(&ctx, &slug, page.unwrap_or(1), sort.as_deref())
        .map_err(|e| e.to_string())
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
    let _ = state;
    let url = format!("https://lewdzone.com/game/{}/", slug.trim_end_matches('/'));
    let html = crate::scraper::fetch(&url).map_err(|e| e.to_string())?;
    Ok(crate::scraper::game::parse_game(&html))
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
    let select = crate::core::download::Select {
        game: &slug,
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
/// never the state lock) resolves go-links and dispatches to the active manager
/// off the webview thread, so the UI stays interactive and more downloads can be
/// scheduled while any are active (Rule 05). `source` (chosen from the game page
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

/// Favorites list (heart-tab). Persistence lands with the library/downloads
/// milestone; surface an empty list for now.
#[tauri::command]
fn favorites_list(_state: tauri::State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    Ok(Vec::new())
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

/// `settings.set(key, value)` — persist and return the new value.
#[tauri::command]
fn settings_set(
    state: tauri::State<'_, AppState>,
    key: String,
    value: String,
) -> Result<SettingView, String> {
    let ctx = state
        .context
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    let value = crate::core::settings::apply(&ctx, &key, &value).map_err(|e| e.to_string())?;
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
    crate::core::settings::apply(&ctx, "theme", effective)?;
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

/// CLI entry point called from `main.rs` when argv has subcommands.
pub fn cli_main() -> std::process::ExitCode {
    use clap::Parser;

    let cli = cli::Cli::parse();
    let code = cli::run(cli);
    std::process::ExitCode::from(code.as_i32() as u8)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (db, config) = default_context();
    let queue = Arc::new(crate::core::queue::Queue::new());
    // Background download worker: owns its Context clone so it never touches the
    // state lock; the webview enqueues via `game_download` and stays responsive.
    let _worker =
        crate::core::queue::spawn_worker(queue.clone(), Context::new(db.clone(), config.clone()));
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            context: Mutex::new(Context::new(db, config)),
            queue,
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
            library_list
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
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
