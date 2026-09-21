//! Theme skins (ADR-0005 retained Steam feature): user-installed skin packages
//! that override the design tokens via CSS custom properties. Skin folders live
//! at `<config_root>/skins/<Name>/` with a `theme.json` manifest and optional
//! `assets/`. Skins may only carry tokens + assets — never scripts (Rule 10);
//! a malformed skin falls back to the built-in default theme.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::paths;
use crate::core::Error;

/// Built-in default design tokens — the lewdzone.com site palette (design
/// tokens in view-designer.md). All keys are `--lz-*` CSS custom properties.
pub fn default_tokens() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("--lz-accent".into(), "#CB3D80".into()), // site magenta accent
        ("--lz-primary".into(), "#BC2A5E".into()), // hot pink (icon two-tone)
        ("--lz-cyan".into(), "#32B6CD".into()),   // cyan (icon two-tone)
        ("--lz-bg".into(), "#14121A".into()),     // near-black purple tint
        ("--lz-surface".into(), "#1F1B28".into()),
        ("--lz-surface-2".into(), "#2A2434".into()),
        ("--lz-text".into(), "#F4F1F6".into()),
        ("--lz-text-dim".into(), "#BDB3C6".into()),
        ("--lz-danger".into(), "#E5484D".into()),
        ("--lz-ok".into(), "#46D88B".into()),
    ])
}

/// Theme tokens are only valid if they are `--lz-*` custom properties.
fn valid_token_key(key: &str) -> bool {
    key.starts_with("--lz-") && !key.is_empty()
}

/// Manifest of a user-installed skin package.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkinManifest {
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub tokens: BTreeMap<String, String>,
}

impl SkinManifest {
    /// Load+validate a skin manifest; `None` on missing file.
    /// Invalid manifests return an error so the caller can fall back.
    pub fn load(path: &Path) -> Result<Option<Self>, Error> {
        if !path.exists() {
            return Ok(None);
        }
        let raw = fs::read_to_string(path)?;
        let manifest: SkinManifest = serde_json::from_str(&raw).map_err(Error::from)?;
        validate(&manifest)?;
        Ok(Some(manifest))
    }

    /// Effective tokens for this skin: defaults overridden by skin tokens.
    /// Keys that fail the `--lz-*` rule are dropped, never applied.
    pub fn effective_tokens(&self) -> BTreeMap<String, String> {
        let mut merged = default_tokens();
        for (k, v) in &self.tokens {
            if valid_token_key(k) {
                merged.insert(k.clone(), v.clone());
            }
        }
        merged
    }
}

/// Validation gate: name matches the folder, tokens are `--lz-*` custom
/// properties only (Rule 10 — no script surfaces in a skin).
pub fn validate(m: &SkinManifest) -> Result<(), Error> {
    if m.name.trim().is_empty() {
        return Err(Error::Usage("skin manifest name is empty".into()));
    }
    if m.name.trim() != m.name {
        return Err(Error::Usage(
            "skin manifest name has surrounding whitespace".into(),
        ));
    }
    for key in m.tokens.keys() {
        if !valid_token_key(key) {
            return Err(Error::Usage(format!(
                "skin token '{key}' is not a --lz-* custom property"
            )));
        }
    }
    Ok(())
}

/// Resolve effective tokens for a skin by folder name; malformed or missing
/// skins fall back to the built-in default (`None` = default theme).
pub fn resolve(skin_name: Option<&str>) -> Result<BTreeMap<String, String>, Error> {
    let Some(name) = skin_name else {
        return Ok(default_tokens());
    };
    let Some(dir) = paths::skin_dir(name) else {
        return Ok(default_tokens());
    };
    match SkinManifest::load(&dir.join("theme.json"))? {
        None => Ok(default_tokens()),
        Some(m) => Ok(m.effective_tokens()),
    }
}

/// List installed skin names (folders under skins dir that carry a valid
/// theme.json). Sorted. The default theme is not listed as a skin.
pub fn installed() -> Result<Vec<String>, Error> {
    let Some(dir) = paths::skins_dir() else {
        return Ok(Vec::new());
    };
    let mut names = Vec::new();
    for entry in fs::read_dir(&dir).map_err(Error::from)? {
        let entry = entry.map_err(Error::from)?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if let Ok(Some(_)) = SkinManifest::load(&path.join("theme.json")) {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    names.sort();
    Ok(names)
}

/// Absolute file path of a skin-managed asset (only files under the skin
/// folder resolve — Rule 10 sandbox). `None` for anything else.
pub fn asset_path(skin_name: &str, relative: &str) -> Option<PathBuf> {
    let dir = paths::skin_dir(skin_name)?;
    let candidate = dir.join(relative);
    if candidate.starts_with(&dir) && candidate.is_file() {
        Some(candidate)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_lz_tokens_only() {
        let tokens = default_tokens();
        assert!(!tokens.is_empty());
        assert!(tokens.keys().all(|k| valid_token_key(k)));
        assert!(tokens.contains_key("--lz-accent"));
        assert!(tokens.contains_key("--lz-bg"));
    }

    #[test]
    fn validate_requires_lz_token_keys() {
        let good = SkinManifest {
            name: "Pink Neon".into(),
            version: "1.0".into(),
            author: "you".into(),
            tokens: BTreeMap::from([("--lz-accent".into(), "#ff00ff".into())]),
        };
        assert!(validate(&good).is_ok());

        let bad = SkinManifest {
            name: "Sketchy".into(),
            version: "1.0".into(),
            author: "you".into(),
            tokens: BTreeMap::from([
                ("--lz-accent".into(), "#f0f".into()),
                ("background".into(), "url(evil)".into()),
            ]),
        };
        assert!(matches!(validate(&bad), Err(Error::Usage(_))));
    }

    #[test]
    fn effective_tokens_merge_and_drop_bad() {
        let m = SkinManifest {
            name: "test".into(),
            version: "0".into(),
            author: String::new(),
            tokens: BTreeMap::from([
                ("--lz-accent".into(), "#ff00ff".into()),
                ("background".into(), "url(evil)".into()), // dropped
            ]),
        };
        let e = m.effective_tokens();
        assert_eq!(e.get("--lz-accent").map(String::as_str), Some("#ff00ff"));
        assert!(!e.contains_key("background"));
    }

    #[test]
    fn resolve_default_when_none() {
        assert_eq!(resolve(None).expect("default"), default_tokens());
    }

    #[test]
    fn resolve_missing_skin_falls_back_to_default() {
        assert_eq!(
            resolve(Some("definitely-not-installed")).expect("fallback"),
            default_tokens()
        );
    }

    #[test]
    fn reject_manifest_with_surrounding_whitespace_name() {
        let m = SkinManifest {
            name: " padded ".into(),
            version: "0".into(),
            author: String::new(),
            tokens: BTreeMap::new(),
        };
        assert!(matches!(validate(&m), Err(Error::Usage(_))));
    }

    #[test]
    fn asset_path_only_resolves_inside_skin_dir() {
        let name = "no-skin-here";
        assert_eq!(asset_path(name, "theme.json"), None);
        assert_eq!(asset_path(name, "../../secrets.txt"), None);
    }
}
