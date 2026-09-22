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
use std::sync::Mutex;

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
}

/// Wire format returned to the Settings page (cluster of a single setting).
#[derive(Debug, Serialize, Deserialize)]
pub struct SettingView {
    pub key: String,
    pub value: serde_json::Value,
}

/// Fetch + parse one storefront archive page. The Store view calls this; the
/// CLI `sync`/`search` subcommands hit the same `core::catalog` function.
#[tauri::command]
fn catalog_page(
    state: tauri::State<'_, AppState>,
    page: Option<u32>,
    sort: Option<String>,
    platform: Option<String>,
) -> Result<crate::scraper::archive::ArchivePage, String> {
    let ctx = state
        .context
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    crate::core::catalog::archive_page(
        &ctx,
        page.unwrap_or(1),
        sort.as_deref(),
        platform.as_deref(),
    )
    .map_err(|e| e.to_string())
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
/// return the new effective tokens so the UI can apply them immediately.
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
        Some("") | None => None,
        Some(n) => Some(n.to_string()),
    };
    let tokens = tokens_for(&ctx)?;
    crate::core::settings::apply(&ctx, "theme", effective.as_deref().unwrap_or(""))?;
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
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            context: Mutex::new(Context::new(db, config)),
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            settings_get,
            settings_set,
            themes_list,
            themes_tokens,
            themes_apply,
            catalog_page
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
