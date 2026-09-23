//! Settings: read/write the local JSON config file (Rule 10 secrets stay out).

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
    /// Grace period between consecutive download starts (seconds); unset = 20.
    pub download_grace_seconds: Option<u64>,
    /// Preferred cloud-source hosts, comma-separated (e.g. `mega,google,dropbox`).
    /// Entries are ordered/filtered by this list before download selection.
    pub source_priority: Option<String>,
    /// Which sidebar tab opens at launch (`store`, `favorites`, `library`,
    /// `downloads`, `settings`); unset = `store`.
    pub home_page: Option<String>,
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
            "download-grace-seconds" => self.download_grace_seconds.map(serde_json::Value::from),
            "source-priority" => self.source_priority.clone().map(serde_json::Value::String),
            "home-page" => self.home_page.clone().map(serde_json::Value::String),
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
                self.library_root = Some(take_string(key, value)?);
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
    "download-grace-seconds",
    "source-priority",
    "home-page",
];

/// All known + extra settings as a JSON object (data form for GUI bridge).
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

/// Load settings at `ctx.config_path` and return the snapshot map.
pub fn load_snapshot(ctx: &Context) -> Result<serde_json::Value, Error> {
    let s = Settings::load(&ctx.config_path)?;
    Ok(snapshot(&s))
}

/// `settings get [key]` — prints one value or the whole map (JSON if `--json`).
pub fn get(ctx: &Context, key: Option<&str>) -> Result<crate::cli::ExitCode, Error> {
    let s = Settings::load(&ctx.config_path)?;
    match key {
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
            println!("{}", serde_json::to_string_pretty(&snapshot(&s))?);
            Ok(crate::cli::ExitCode::Ok)
        }
    }
}

/// `settings set <key> <value>` — persists a value; secrets validated only.
pub fn set(ctx: &Context, key: &str, value: &str) -> Result<crate::cli::ExitCode, Error> {
    let parsed = apply(ctx, key, value)?;
    println!("{key} = {parsed}");
    Ok(crate::cli::ExitCode::Ok)
}

/// Persist a setting and return the new value (data form for the GUI bridge).
pub fn apply(ctx: &Context, key: &str, value: &str) -> Result<serde_json::Value, Error> {
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
        let parsed = apply(&ctx, "source-priority", "").expect("empty value applies");
        assert_eq!(parsed, serde_json::Value::String(String::new()));
        let saved = Settings::load(&ctx.config_path).expect("load");
        assert_eq!(saved.source_priority.as_deref(), Some(""));
        let _ = fs::remove_dir_all(ctx.config_path.parent().expect("dir"));
    }

    #[test]
    fn apply_parses_digits_booleans_and_strings() {
        let ctx = tmp_ctx("kinds");
        assert_eq!(
            apply(&ctx, "download-grace-seconds", "30").expect("number"),
            serde_json::Value::Number(serde_json::Number::from(30u64))
        );
        assert_eq!(
            apply(&ctx, "capture-aware", "true").expect("bool"),
            serde_json::Value::Bool(true)
        );
        assert_eq!(
            apply(&ctx, "theme", "Nord").expect("string"),
            serde_json::Value::String("Nord".into())
        );
        // Non-numeric strings must not be rejected as numbers.
        assert!(matches!(
            apply(&ctx, "download-grace-seconds", "abc"),
            Err(Error::Usage(_))
        ));
        let _ = fs::remove_dir_all(ctx.config_path.parent().expect("dir"));
    }
}
