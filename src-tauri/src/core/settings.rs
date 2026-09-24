//! Settings: read/write the local JSON config file (Rule 10 secrets stay out).
//!
//! Secret keys (API keys) live in the SQLite DB's `secret` table, never here.
//! `set(.., secret: true)` / `apply(.., secret: true)` route to the DB and are
//! never echoed — `get` prints presence only ("(set)"/"(not set)").

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::{Context, Error};

/// Runtime settings. Keys are documented in wiki/Configuration.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Settings {
    /// Root folder where downloaded games land.
    pub download_root: Option<String>,
    /// Content-provider priority order (comma-separated).
    pub content_priority: Option<String>,
    /// Enable installer capture heuristics.
    pub capture_aware: Option<bool>,
    /// Active theme skin name (ADR-0005); unset = built-in default.
    pub theme: Option<String>,
    /// Library root override (library-folders analog, ADR-0005);
    /// unset = `<data_root>/library`.
    pub library_root: Option<String>,
    /// Directory where the user extracts their games, scanned to populate the library.
    pub games_dir: Option<String>,
    /// Grace period between consecutive download starts (seconds); unset = 20.
    pub download_grace_seconds: Option<u64>,
    /// Preferred cloud-source hosts, comma-separated (e.g. `mega,google,dropbox`).
    /// Entries are ordered/filtered by this list before download selection.
    pub source_priority: Option<String>,
    /// Which sidebar tab opens at launch (`store`, `favorites`, `library`,
    /// `downloads`, `settings`); unset = `store`.
    pub home_page: Option<String>,
    /// Path to 7-Zip CLI console executable (7za.exe / 7z.exe / 7za).
    pub seven_zip_path: Option<String>,
    /// Extra user-provided keys, kept un-echoed (Rule 10).
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl Settings {
    pub fn load(path: &Path) -> Result<Self, Error> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(path)?;
        let parsed = serde_json::from_str(&raw).map_err(Error::from)?;
        Ok(parsed)
    }

    pub fn save(&self, path: &Path) -> Result<(), Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn get_value(&self, key: &str) -> Option<serde_json::Value> {
        let s = match key {
            "download-root" => self.download_root.clone().map(serde_json::Value::String),
            "content-priority" => self.content_priority.clone().map(serde_json::Value::String),
            "capture-aware" => self.capture_aware.map(serde_json::Value::Bool),
            "theme" => self.theme.clone().map(serde_json::Value::String),
            "library-root" => self.library_root.clone().map(serde_json::Value::String),
            "games-dir" => self.games_dir.clone().map(serde_json::Value::String),
            "download-grace-seconds" => self.download_grace_seconds.map(serde_json::Value::from),
            "source-priority" => self.source_priority.clone().map(serde_json::Value::String),
            "home-page" => self.home_page.clone().map(serde_json::Value::String),
            "7z-path" | "seven-zip-path" => {
                self.seven_zip_path.clone().map(serde_json::Value::String)
            }
            other => self.extra.get(other).cloned(),
        };
        s
    }

    pub fn set_value(&mut self, key: &str, value: serde_json::Value) -> Result<(), Error> {
        match key {
            "download-root" => {
                self.download_root = Some(take_string(key, value)?);
            }
            "content-priority" => {
                self.content_priority = Some(take_string(key, value)?);
            }
            "capture-aware" => {
                self.capture_aware = Some(take_bool(key, value)?);
            }
            "theme" => {
                self.theme = Some(take_string(key, value)?);
            }
            "library-root" => {
                let v = take_string(key, value)?;
                if !v.trim().is_empty() {
                    let _ = crate::core::folder::initialize_library_structure(Path::new(&v));
                }
                self.library_root = Some(v);
            }
            "games-dir" => {
                let v = take_string(key, value)?;
                if !v.trim().is_empty() {
                    let _ = crate::core::folder::initialize_library_structure(Path::new(&v));
                }
                self.games_dir = Some(v);
            }
            "download-grace-seconds" => {
                self.download_grace_seconds = Some(take_u64(key, value)?);
            }
            "source-priority" => {
                self.source_priority = Some(take_string(key, value)?);
            }
            "home-page" => {
                self.home_page = Some(take_string(key, value)?);
            }
            "7z-path" | "seven-zip-path" => {
                self.seven_zip_path = Some(take_string(key, value)?);
            }
            other => {
                self.extra.insert(other.to_string(), value);
            }
        }
        Ok(())
    }
}

fn take_string(key: &str, value: serde_json::Value) -> Result<String, Error> {
    value
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| Error::Usage(format!("setting '{key}' expects a string")))
}

fn take_bool(key: &str, value: serde_json::Value) -> Result<bool, Error> {
    value
        .as_bool()
        .ok_or_else(|| Error::Usage(format!("setting '{key}' expects a boolean")))
}

fn take_u64(key: &str, value: serde_json::Value) -> Result<u64, Error> {
    value
        .as_u64()
        .ok_or_else(|| Error::Usage(format!("setting '{key}' expects a number")))
}

const KNOWN_KEYS: &[&str] = &[
    "download-root",
    "content-priority",
    "capture-aware",
    "theme",
    "library-root",
    "games-dir",
    "download-grace-seconds",
    "source-priority",
    "home-page",
    "7z-path",
];

/// Keys whose values are secrets: stored in the SQLite `secret` table, never
/// in the JSON config, and never echoed back (Rule 10).
pub const SECRET_KEYS: &[&str] = &["sgdb-api-key", "igdb-client-id", "igdb-client-secret"];

/// Is this key a secret (API key)? Secrets bypass the JSON config entirely.
pub fn is_secret_key(key: &str) -> bool {
    SECRET_KEYS.contains(&key)
}

/// All known + extra settings as a JSON object (data form for GUI bridge).
/// Secrets are omitted here — the bridge reads their presence separately.
pub fn snapshot(s: &Settings) -> serde_json::Value {
    let mut all = serde_json::Map::new();
    for k in KNOWN_KEYS {
        if let Some(v) = s.get_value(k) {
            all.insert((*k).to_string(), v);
        }
    }
    for (k, v) in &s.extra {
        all.insert(k.clone(), v.clone());
    }
    serde_json::Value::Object(all)
}

/// Load settings at `ctx.config_path` and return the snapshot map augmented
/// with secret-key presence markers ("(set)"/"(not set)") for the UI.
pub fn load_snapshot(ctx: &Context) -> Result<serde_json::Value, Error> {
    let s = Settings::load(&ctx.config_path)?;
    let mut all = snapshot(&s);
    if let serde_json::Value::Object(map) = &mut all {
        for k in SECRET_KEYS {
            let present = secret_present(ctx, k)?;
            map.insert(
                (*k).to_string(),
                serde_json::Value::String(if present {
                    "(set)".into()
                } else {
                    "(not set)".into()
                }),
            );
        }
    }
    Ok(all)
}

/// Read a secret's presence (never its value) from the SQLite DB.
pub fn secret_present(ctx: &Context, key: &str) -> Result<bool, Error> {
    let conn = crate::db::open(&ctx.db_path)?;
    crate::db::migrate(&conn)?;
    Ok(crate::db::repo::secret_get(&conn, key)?.is_some())
}

/// `settings get [key]` — prints one value or the whole map (JSON if `--json`).
/// Secret keys print presence only, never the stored value.
pub fn get(ctx: &Context, key: Option<&str>) -> Result<crate::cli::ExitCode, Error> {
    let s = Settings::load(&ctx.config_path)?;
    match key {
        Some(k) if is_secret_key(k) => {
            let present = secret_present(ctx, k)?;
            println!("{k} = {}", if present { "(set)" } else { "(not set)" });
            Ok(crate::cli::ExitCode::Ok)
        }
        Some(k) if !KNOWN_KEYS.contains(&k) && !s.extra.contains_key(k) => {
            Err(Error::Usage(format!("unknown setting '{k}'")))
        }
        Some(k) => {
            if let Some(v) = s.get_value(k) {
                println!("{k} = {v}");
            }
            Ok(crate::cli::ExitCode::Ok)
        }
        None => {
            println!("{}", serde_json::to_string_pretty(&load_snapshot(ctx)?)?);
            Ok(crate::cli::ExitCode::Ok)
        }
    }
}

/// `settings set <key> <value> [--secret]` — persists a value; secrets are
/// written to the SQLite DB and never echoed.
pub fn set(
    ctx: &Context,
    key: &str,
    value: &str,
    secret: bool,
) -> Result<crate::cli::ExitCode, Error> {
    if secret || is_secret_key(key) {
        let conn = crate::db::open(&ctx.db_path)?;
        crate::db::migrate(&conn)?;
        crate::db::repo::secret_set(&conn, key, value)?;
        println!(
            "{key} = {}",
            if value.is_empty() {
                "(cleared)"
            } else {
                "(set)"
            }
        );
        return Ok(crate::cli::ExitCode::Ok);
    }
    let parsed = apply(ctx, key, value, false)?;
    println!("{key} = {parsed}");
    Ok(crate::cli::ExitCode::Ok)
}

/// Persist a setting and return the new value (data form for the GUI bridge).
/// With `secret: true` the value goes to the SQLite `secret` table and the
/// returned value is a presence marker, never the raw secret.
pub fn apply(
    ctx: &Context,
    key: &str,
    value: &str,
    secret: bool,
) -> Result<serde_json::Value, Error> {
    if secret || is_secret_key(key) {
        let conn = crate::db::open(&ctx.db_path)?;
        crate::db::migrate(&conn)?;
        crate::db::repo::secret_set(&conn, key, value)?;
        return Ok(serde_json::Value::String(if value.is_empty() {
            "(cleared)".into()
        } else {
            "(set)".into()
        }));
    }
    let mut s = Settings::load(&ctx.config_path)?;
    let parsed: serde_json::Value = if value == "true" || value == "false" {
        serde_json::Value::Bool(value == "true")
    } else if value.is_empty() {
        // Empty text — a cleared setting — is a string, not a number.
        serde_json::Value::String(String::new())
    } else if value.chars().all(|c| c.is_ascii_digit()) {
        serde_json::Value::Number(
            value
                .parse::<u64>()
                .map(serde_json::Number::from)
                .map_err(|e| Error::Usage(format!("setting '{key}': {e}")))?,
        )
    } else {
        serde_json::Value::String(value.to_string())
    };
    s.set_value(key, parsed.clone())?;
    s.save(&ctx.config_path)?;
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_ctx(tag: &str) -> Context {
        let dir = std::env::temp_dir().join(format!("lz-settings-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("temp dir");
        Context::new(dir.join("test.db"), dir.join("config.json"))
    }

    #[test]
    fn apply_empty_value_parses_as_string_not_number() {
        let ctx = tmp_ctx("empty");
        // Regression: "" is all-()digits vacuously; it must not panic on parse.
        let parsed = apply(&ctx, "source-priority", "", false).expect("empty value applies");
        assert_eq!(parsed, serde_json::Value::String(String::new()));
        let saved = Settings::load(&ctx.config_path).expect("load");
        assert_eq!(saved.source_priority.as_deref(), Some(""));
        let _ = fs::remove_dir_all(ctx.config_path.parent().expect("dir"));
    }

    #[test]
    fn apply_parses_digits_booleans_and_strings() {
        let ctx = tmp_ctx("kinds");
        assert_eq!(
            apply(&ctx, "download-grace-seconds", "30", false).expect("number"),
            serde_json::Value::Number(serde_json::Number::from(30u64))
        );
        assert_eq!(
            apply(&ctx, "capture-aware", "true", false).expect("bool"),
            serde_json::Value::Bool(true)
        );
        assert_eq!(
            apply(&ctx, "theme", "Nord", false).expect("string"),
            serde_json::Value::String("Nord".into())
        );
        // Non-numeric strings must not be rejected as numbers.
        assert!(matches!(
            apply(&ctx, "download-grace-seconds", "abc", false),
            Err(Error::Usage(_))
        ));
        let _ = fs::remove_dir_all(ctx.config_path.parent().expect("dir"));
    }

    #[test]
    fn secrets_go_to_sqlite_and_are_never_echoed() {
        let ctx = tmp_ctx("secrets");
        assert_eq!(
            apply(&ctx, "sgdb-api-key", "super-secret", false).expect("auto-secret route"),
            serde_json::Value::String("(set)".into())
        );
        assert_eq!(
            apply(&ctx, "igdb-client-secret", "client-secret", true).expect("secret flag"),
            serde_json::Value::String("(set)".into())
        );
        // The JSON config must not contain the secret value.
        let saved = Settings::load(&ctx.config_path).expect("load");
        assert!(!saved.extra.contains_key("sgdb-api-key"));
        assert!(!saved.extra.contains_key("igdb-client-secret"));

        // get() prints presence only, not the value.
        assert!(secret_present(&ctx, "sgdb-api-key").unwrap());
        assert!(secret_present(&ctx, "igdb-client-secret").unwrap());
        assert!(!secret_present(&ctx, "igdb-client-id").unwrap());

        // Clearing returns a marker and removes the row.
        assert_eq!(
            apply(&ctx, "sgdb-api-key", "", false).expect("clear secret"),
            serde_json::Value::String("(cleared)".into())
        );
        assert!(!secret_present(&ctx, "sgdb-api-key").unwrap());

        let _ = fs::remove_dir_all(ctx.config_path.parent().expect("dir"));
    }
}
