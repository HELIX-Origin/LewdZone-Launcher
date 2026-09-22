//! Shared Rust core — the engine behind BOTH the Tauri app commands (invoked
//! from the webview) and the native CLI subcommands (Rule 03, Rule 13).
//!
//! Nothing in here knows about windows, webviews, or argv parsing. Webview
//! handlers wrap these functions as `#[tauri::command]`s; the CLI wraps them
//! as subcommands. App == CLI by construction.

pub mod catalog;
pub mod dm;
pub mod download;
pub mod info;
pub mod launch;
pub mod library;
pub mod list;
pub mod models;
pub mod paths;
pub mod search;
pub mod settings;
pub mod shortcuts;
pub mod skins;
pub mod sync;

use std::path::PathBuf;

/// Typed error carrying the stable exit-code contract (Rule 12).
#[derive(Debug)]
pub enum Error {
    /// Usage/bad input (exit 2).
    Usage(String),
    /// Network/site error (exit 3).
    Network(String),
    /// Required download manager missing (exit 4).
    DmMissing(String),
    /// User interrupted (exit 5).
    Interrupted,
    /// Everything else (exit 1).
    Runtime(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage(msg) => write!(f, "{msg}"),
            Self::Network(msg) => write!(f, "network error: {msg}"),
            Self::DmMissing(msg) => write!(f, "download manager missing: {msg}"),
            Self::Interrupted => write!(f, "interrupted"),
            Self::Runtime(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Runtime(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Runtime(format!("invalid JSON: {err}"))
    }
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Self {
        Self::Runtime(format!("sqlite: {err}"))
    }
}

impl From<Error> for String {
    fn from(err: Error) -> Self {
        err.to_string()
    }
}

/// Everything a command needs to do its job: resolved tool paths.
#[derive(Clone, Debug)]
pub struct Context {
    /// SQLite catalog path.
    pub db_path: PathBuf,
    /// JSON config file path.
    pub config_path: PathBuf,
}

impl Context {
    pub fn new(db_path: PathBuf, config_path: PathBuf) -> Self {
        Self {
            db_path,
            config_path,
        }
    }
}

/// Marker for domain functions not yet implemented (Phase 2 engine).
pub fn not_yet(cmd: &str) -> Error {
    Error::Runtime(format!(
        "'{cmd}' is not yet implemented in this build — see ROADMAP Phase 2"
    ))
}
