//! Per-OS default locations for the SQLite catalog, JSON config, and the
//! Steam-mirror library tree (Rule 02, ADR-0005).

use std::path::PathBuf;

use crate::core::Error;

const DATA_DIR: &str = "lewdzone-launcher";

/// Default SQLite catalog path per OS:
/// - Windows: `%APPDATA%\lewdzone-launcher\lewdzone.db`
/// - macOS:   `~/Library/Application Support/lewdzone-launcher/lewdzone.db`
/// - Linux:   `$XDG_DATA_HOME/lewdzone-launcher/lewdzone.db` (`~/.local/share/...`)
pub fn default_db_path() -> Option<PathBuf> {
    base_data_dir().map(|base| base.join("lewdzone.db"))
}

/// Default config file path per OS:
/// - Windows: `%APPDATA%\lewdzone-launcher\config.json`
/// - macOS:   `~/Library/Application Support/lewdzone-launcher/config.json`
/// - Linux:   `$XDG_CONFIG_HOME/lewdzone-launcher/config.json` (`~/.config/...`)
pub fn default_config_path() -> Option<PathBuf> {
    base_config_dir().map(|base| base.join("config.json"))
}

/// App data root ("steam root", ADR-0005):
/// - Windows: `%APPDATA%\lewdzone-launcher`
/// - macOS:   `~/Library/Application Support/lewdzone-launcher`
/// - Linux:   `$XDG_DATA_HOME/lewdzone-launcher` (`~/.local/share/...`)
pub fn data_root() -> Option<PathBuf> {
    base_data_dir()
}

/// App config root:
/// - Windows/macOS share the data root; Linux uses XDG config home.
pub fn config_root() -> Option<PathBuf> {
    base_config_dir()
}

// Steam-mirror tree (ADR-0005): all under `<data_root>/`.
//   lewdzone.db, config.json, appcache/, logs/, library/, userdata/
pub fn appcache_dir() -> Option<PathBuf> {
    data_root().map(|root| root.join("appcache"))
}

pub fn logs_dir() -> Option<PathBuf> {
    data_root().map(|root| root.join("logs"))
}

/// The "steamapps/" analog — holds libraryfolders.json, app manifests,
/// `common/<Title>/` installs, `downloading/`, and `artwork/`.
pub fn library_dir() -> Option<PathBuf> {
    data_root().map(|root| root.join("library"))
}

/// Installed games: `<library>/common/<Game Title>/`.
pub fn library_common_dir() -> Option<PathBuf> {
    library_dir().map(|lib| lib.join("common"))
}

/// In-progress downloads: `<library>/downloading/<post_id>/`.
pub fn library_downloading_dir() -> Option<PathBuf> {
    library_dir().map(|lib| lib.join("downloading"))
}

/// Grid artwork cache: `<library>/artwork/<post_id>_<kind>.png`.
pub fn library_artwork_dir() -> Option<PathBuf> {
    library_dir().map(|lib| lib.join("artwork"))
}

/// The per-game manifest index (`libraryfolders.json` analog).
pub fn libraryfolders_path() -> Option<PathBuf> {
    library_dir().map(|lib| lib.join("libraryfolders.json"))
}

/// Per-game manifest: `<library>/appmanifest_<post_id>.json`.
pub fn appmanifest_path(post_id: i64) -> Option<PathBuf> {
    library_dir().map(|lib| lib.join(format!("appmanifest_{post_id}.json")))
}

/// Per-user profile dir (`userdata/<local_user_id>/` legacy).
pub fn userdata_dir() -> Option<PathBuf> {
    data_root().map(|root| root.join("userdata"))
}

/// Machine-local caches (Steam `%LOCALAPPDATA%\Steam\` analog):
/// - Windows: `%LOCALAPPDATA%\lewdzone-launcher\htmlcache` etc.
/// - macOS:   `~/Library/Caches/lewdzone-launcher`
/// - Linux:   `$XDG_CACHE_HOME/lewdzone-launcher` (`~/.cache/...`)
pub fn cache_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .map(|b| b.join(DATA_DIR))
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|h| h.join("Library").join("Caches").join(DATA_DIR))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
            .map(|b| b.join(DATA_DIR))
    }
    #[cfg(not(any(unix, windows, target_os = "macos")))]
    {
        let _ = DATA_DIR;
        None
    }
}

/// Webview/cache folder under the cache dir (Steam `htmlcache` analog).
pub fn htmlcache_dir() -> Option<PathBuf> {
    cache_dir().map(|c| c.join("htmlcache"))
}

/// Per-game save area (Steam `Documents\My Games\<Game>\` analog).
/// Falls back under the data root when no Documents folder resolves.
pub fn my_games_dir() -> Option<PathBuf> {
    let documents = documents_dir();
    documents.or_else(data_root)
}

fn documents_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("USERPROFILE")
            .map(PathBuf::from)
            .map(|p| p.join("Documents").join("My Games"))
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|h| h.join("Documents").join("My Games"))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|h| h.join("Documents").join("My Games"))
    }
    #[cfg(not(any(unix, windows, target_os = "macos")))]
    {
        None
    }
}

/// Per-game save folder: `<my_games_dir>/<Game Title>/`.
pub fn game_saves_dir(title: &str) -> Option<PathBuf> {
    my_games_dir().map(|dir| dir.join(sanitize_dir_name(title)))
}

/// User-installed theme skins (retained Steam feature, ADR-0005):
/// `<config_root>/skins/<Name>/theme.json` + token overrides.
pub fn skins_dir() -> Option<PathBuf> {
    config_root().map(|root| root.join("skins"))
}

/// Resolve a skin folder by name; `None` name means the built-in default.
pub fn skin_dir(name: &str) -> Option<PathBuf> {
    skins_dir().map(|dir| dir.join(sanitize_dir_name(name)))
}

/// Strip characters unsafe in folder names while keeping the Steam-like title.
fn sanitize_dir_name(title: &str) -> String {
    let bad = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let clean: String = title
        .chars()
        .map(|c| if bad.contains(&c) { '_' } else { c })
        .collect();
    let trimmed = clean.trim();
    if trimmed.is_empty() {
        "Game".to_string()
    } else {
        trimmed.to_string()
    }
}

fn base_data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .map(|b| b.join(DATA_DIR))
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|h| h.join("Library").join("Application Support").join(DATA_DIR))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local").join("share"))
            })
            .map(|b| b.join(DATA_DIR))
    }
    #[cfg(not(any(unix, windows, target_os = "macos")))]
    {
        let _ = DATA_DIR;
        None
    }
}

fn base_config_dir() -> Option<PathBuf> {
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        base_data_dir()
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
            .map(|b| b.join(DATA_DIR))
    }
    #[cfg(not(any(unix, windows, target_os = "macos")))]
    {
        let _ = DATA_DIR;
        None
    }
}

/// Validate a user-supplied override path exists/usable.
pub fn require_override(path: &str, label: &str) -> Result<PathBuf, Error> {
    let p = PathBuf::from(path);
    if p.as_os_str().is_empty() {
        return Err(Error::Usage(format!("empty {label} path")));
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_an_absolute_default_db_path() {
        let p = default_db_path().expect("env has a home");
        assert!(p.is_absolute());
        assert!(p.ends_with("lewdzone.db"));
    }

    #[test]
    fn returns_an_absolute_default_config_path() {
        let p = default_config_path().expect("env has a home");
        assert!(p.is_absolute());
        assert!(p.ends_with("config.json"));
    }

    #[test]
    fn rejects_empty_override() {
        assert!(matches!(require_override("", "db"), Err(Error::Usage(_))));
    }

    #[test]
    fn data_root_holds_catalog_and_library() {
        let root = data_root().expect("data root resolves");
        assert!(root.is_absolute());
        assert_eq!(root.join("lewdzone.db"), default_db_path().expect("db"));
        assert!(appcache_dir().expect("appcache").starts_with(&root));
        assert!(logs_dir().expect("logs").starts_with(&root));
        assert!(library_dir().expect("library").starts_with(&root));
        assert!(library_common_dir().expect("common").starts_with(&root));
        assert!(library_artwork_dir().expect("artwork").starts_with(&root));
        assert!(libraryfolders_path().expect("folders").starts_with(&root));
    }

    #[test]
    fn appmanifest_path_takes_post_id() {
        assert_eq!(
            appmanifest_path(1234).expect("manifest"),
            library_dir()
                .expect("library")
                .join("appmanifest_1234.json")
        );
    }

    #[test]
    fn cache_and_saves_resolve() {
        assert!(cache_dir().expect("cache").is_absolute());
        assert!(htmlcache_dir()
            .expect("htmlcache")
            .starts_with(&cache_dir().expect("cache")));
        assert!(my_games_dir().expect("my games").is_absolute());
        assert!(game_saves_dir("Treasure of Nadia")
            .expect("saves")
            .ends_with("Treasure of Nadia"));
    }

    #[test]
    fn game_saves_dir_sanitizes_unsafe_names() {
        assert_eq!(sanitize_dir_name("A:B <bad> title?"), "A_B _bad_ title_");
        assert_eq!(sanitize_dir_name("   "), "Game");
        assert_eq!(sanitize_dir_name("ok-name!"), "ok-name!");
    }
}
