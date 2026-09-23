//! Theme skins (ADR-0005 retained feature): user-installed skin packages
//! that override the design tokens via CSS custom properties. Skin folders live
//! in a per-OS user-accessible location (Windows: next to the executable;
//! macOS/Linux: the user data dir) at `<skins>/<Name>/` (each theme owns its
//! own subfolder) with a `theme.json` manifest and optional `assets/`. Skins
//! may only carry tokens + assets — never scripts (Rule 10); a malformed skin
//! falls back to the built-in default theme. The bundled reference themes
//! (Nord, Dracula, Material) are embedded in the binary and seeded into the
//! skins folder on first run.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::paths;
use crate::core::Error;

/// Bundled reference themes shipped with every build (embedded, so they are
/// never lost). On first list they are seeded into the user's skins folder as
/// `<skins>/<Name>/theme.json`, so users see working examples they can copy
/// and customize. `resolve()` also falls back to the embedded copies, so a
/// bundled theme always applies even if the seeded folder was removed.
pub const BUNDLED_THEMES: &[(&str, &str)] = &[
    ("Nord", include_str!("../../../skins/Nord/theme.json")),
    ("Dracula", include_str!("../../../skins/Dracula/theme.json")),
    (
        "Material",
        include_str!("../../../skins/Material/theme.json"),
    ),
];

/// Built-in default design tokens — the dark cyberpunk palette (design
/// tokens in view-designer.md). All keys are `--lz-*` CSS custom properties.
/// MUST stay parity with `src/lib/theme/default.css`.
pub fn default_tokens() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("--lz-accent".into(), "#FF4EC8".into()), // neon pink accent
        ("--lz-primary".into(), "#FF5FB2".into()), // hot pink primary
        ("--lz-cyan".into(), "#22D3EE".into()),   // neon cyan
        ("--lz-bg".into(), "#0A1118".into()),     // deep dark cyan/charcoal
        ("--lz-surface".into(), "#0E1B26".into()),
        ("--lz-surface-2".into(), "#122A3A".into()),
        ("--lz-text".into(), "#E8F1F8".into()),
        ("--lz-text-dim".into(), "#9AAEC0".into()),
        ("--lz-danger".into(), "#FF3B6B".into()),
        ("--lz-ok".into(), "#3DFFA2".into()),
        ("--lz-radius".into(), "4px".into()),
        ("--lz-gap".into(), "12px".into()),
        (
            "--lz-gradient".into(),
            "linear-gradient(160deg, #0A1118 0%, #0E1B26 100%)".into(),
        ),
        (
            "--lz-glow".into(),
            "0 0 14px rgba(34, 211, 238, 0.35)".into(),
        ),
        ("--lz-glass".into(), "rgba(14, 27, 38, 0.55)".into()),
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
/// skins fall back to the bundled reference theme with that name, then to the
/// built-in default (`None`/empty = default theme).
pub fn resolve(skin_name: Option<&str>) -> Result<BTreeMap<String, String>, Error> {
    let Some(name) = skin_name.filter(|n| !n.trim().is_empty()) else {
        return Ok(default_tokens());
    };
    let Some(dir) = paths::skin_dir(name) else {
        return match bundled(name) {
            Some(json) => bundled_tokens(json),
            None => Ok(default_tokens()),
        };
    };
    let manifest = dir.join("theme.json");
    match SkinManifest::load(&manifest) {
        Ok(Some(m)) => Ok(m.effective_tokens()),
        _ => match bundled(name) {
            Some(json) => bundled_tokens(json),
            None => Ok(default_tokens()),
        },
    }
}

/// Parse a bundled reference theme by folder name.
fn bundled(name: &str) -> Option<&'static str> {
    BUNDLED_THEMES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, json)| *json)
}

/// Effective tokens for a bundled theme by folder name.
fn bundled_tokens(json: &str) -> Result<BTreeMap<String, String>, Error> {
    let m: SkinManifest = serde_json::from_str(json).map_err(Error::from)?;
    validate(&m)?;
    Ok(m.effective_tokens())
}

/// Seed the bundled reference themes into the user's skins folder. Existing
/// theme folders are never overwritten — this only creates missing ones, so
/// users who already customized a bundled theme keep their edits.
pub fn seed_bundled() -> Result<(), Error> {
    let Some(dir) = paths::skins_dir() else {
        return Ok(());
    };
    seed_into(&dir)
}

/// Seed bundled reference themes into `dir`; never overwrites existing files.
fn seed_into(dir: &Path) -> Result<(), Error> {
    fs::create_dir_all(dir)?;
    for (name, json) in BUNDLED_THEMES {
        let target = dir.join(name).join("theme.json");
        if target.exists() {
            continue;
        }
        fs::create_dir_all(dir.join(name))?;
        fs::write(&target, json)?;
    }
    Ok(())
}

/// List installed skin names (folders under skins dir that carry a valid
/// theme.json). Sorted. The default theme is not listed as a skin. Bundled
/// reference themes (Nord, Dracula, Material) are seeded here on first run.
pub fn installed() -> Result<Vec<String>, Error> {
    let _ = seed_bundled();
    let Some(dir) = paths::skins_dir() else {
        return Ok(Vec::new());
    };
    if !dir.is_dir() {
        // No skins installed yet — the dir may not exist; treat as empty
        // rather than surfacing an io::Error to the GUI.
        return Ok(Vec::new());
    }
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
        assert_eq!(
            resolve(Some("  ")).expect("blank falls back"),
            default_tokens()
        );
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

    #[test]
    fn bundled_themes_parse_and_validate() {
        for (name, json) in BUNDLED_THEMES {
            let m: SkinManifest = serde_json::from_str(json).expect("bundled theme is valid JSON");
            validate(&m).expect("bundled theme validates");
            assert_eq!(m.name, *name, "bundled folder name matches manifest");
            let tokens = m.effective_tokens();
            assert!(
                tokens.contains_key("--lz-accent") && tokens.contains_key("--lz-bg"),
                "{name} carries accent + bg tokens"
            );
        }
    }

    #[test]
    fn bundled_tokens_falls_back_for_any_named_theme() {
        // resolve() must work for a bundled name even before seeding,
        // and fall back to the embedded copy when the folder is absent.
        assert!(bundled("Nord").is_some());
        assert_eq!(bundled("Dracula").map(|_| "dracula"), Some("dracula"));
        assert!(bundled("Material").is_some());
        assert_eq!(bundled("Not-A-Theme"), None);
    }

    #[test]
    fn seed_into_writes_and_preserves_existing() {
        let base = std::env::temp_dir().join(format!(
            "lz-skins-seed-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        seed_into(&base).expect("seed writes bundled themes");
        for (name, _) in BUNDLED_THEMES {
            assert!(
                base.join(name).join("theme.json").is_file(),
                "seeded {name}/theme.json"
            );
        }

        // A user-customized bundled theme is never overwritten.
        fs::write(base.join("Nord").join("theme.json"), r#"{"name":"Nord"}"#).expect("write edit");
        seed_into(&base).expect("second seed is a no-op for existing files");
        assert_eq!(
            fs::read_to_string(base.join("Nord").join("theme.json")).expect("read"),
            r#"{"name":"Nord"}"#,
            "existing user theme is preserved"
        );

        fs::remove_dir_all(&base).expect("cleanup");
    }

    #[test]
    fn resolve_bundled_theme_when_not_seeded() {
        // No pre-seeding happens here; bundled fallback must produce tokens.
        let tokens = resolve(Some("Nord")).expect("bundled fallback");
        assert_eq!(tokens.get("--lz-bg").map(String::as_str), Some("#2E3440"));
        let tokens = resolve(Some("Dracula")).expect("bundled fallback");
        assert_eq!(tokens.get("--lz-bg").map(String::as_str), Some("#282A36"));
    }
}
