//! Library folder registry + per-game manifests — the Steam `steamapps/`
//! analog (ADR-0005). `libraryfolders.json` mirrors Steam's
//! `libraryfolders.vdf`; `appmanifest_<post_id>.json` mirrors `appmanifest_*.acf`.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::paths;
use crate::core::settings::Settings;
use crate::core::Error;

const DEFAULT_LIBRARY_LABEL: &str = "Default";

/// Ordered library roots: `{ index: LibraryFolder }` (Steam vdf order analog).
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(transparent)]
pub struct LibraryFolders {
    pub folders: BTreeMap<u32, LibraryFolder>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct LibraryFolder {
    pub path: String,
    pub label: Option<String>,
    /// `games: { post_id: size_on_disk }` (Steam `apps` analog).
    #[serde(default)]
    pub games: BTreeMap<i64, u64>,
}

/// `appmanifest_<post_id>.json` — AppState analog.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppManifest {
    pub post_id: i64,
    pub title: String,
    /// Install folder name under `common/` (Steam `installdir` analog).
    pub installdir: String,
    pub version_label: Option<String>,
    pub platform: Option<String>,
    pub tab: Option<String>,
    pub host_slug: Option<String>,
    /// StateFlags analog: 4 = fully installed.
    #[serde(default = "default_state_flags")]
    pub state_flags: u32,
    pub last_updated: Option<String>,
    pub last_played: Option<String>,
    pub size_on_disk: u64,
    #[serde(default)]
    pub bytes_to_download: u64,
    #[serde(default)]
    pub bytes_downloaded: u64,
}

impl Default for AppManifest {
    fn default() -> Self {
        Self {
            post_id: 0,
            title: String::new(),
            installdir: String::new(),
            version_label: None,
            platform: None,
            tab: None,
            host_slug: None,
            state_flags: default_state_flags(),
            last_updated: None,
            last_played: None,
            size_on_disk: 0,
            bytes_to_download: 0,
            bytes_downloaded: 0,
        }
    }
}

fn default_state_flags() -> u32 {
    4
}

impl LibraryFolders {
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

    /// Ensure at least one library root exists; returns the default root.
    pub fn ensure_default(&mut self) -> Result<PathBuf, Error> {
        if self.folders.is_empty() {
            let root = paths::library_dir().ok_or_else(|| {
                Error::Runtime("cannot resolve library folder (no app data dir)".to_string())
            })?;
            self.folders.insert(
                0,
                LibraryFolder {
                    path: root.to_string_lossy().into_owned(),
                    label: Some(DEFAULT_LIBRARY_LABEL.to_string()),
                    games: BTreeMap::new(),
                },
            );
        }
        Ok(PathBuf::from(&self.folders[&0].path))
    }

    /// Record a game in the first library root, updating its size-on-disk.
    pub fn assign_game(&mut self, post_id: i64, size: u64) {
        let Some(first_key) = self.folders.keys().next().copied() else {
            return;
        };
        self.folders
            .entry(first_key)
            .or_default()
            .games
            .insert(post_id, size);
    }

    pub fn iter_games(&self) -> impl Iterator<Item = (&LibraryFolder, i64, u64)> {
        self.folders.values().flat_map(|folder| {
            folder
                .games
                .iter()
                .map(move |(post_id, size)| (folder, *post_id, *size))
        })
    }
}

impl AppManifest {
    /// Load a manifest by post_id; missing file => Ok(None) (not installed).
    pub fn load_by_id(post_id: i64) -> Result<Option<Self>, Error> {
        let path = paths::appmanifest_path(post_id)
            .ok_or_else(|| Error::Runtime("no app data dir".to_string()))?;
        Self::load_from(&path)
    }

    pub fn load_from(path: &Path) -> Result<Option<Self>, Error> {
        if !path.exists() {
            return Ok(None);
        }
        let raw = fs::read_to_string(path)?;
        let parsed = serde_json::from_str(&raw).map_err(Error::from)?;
        Ok(Some(parsed))
    }

    /// Atomically write the manifest next to the install folder.
    pub fn write(&self) -> Result<(), Error> {
        let path = paths::appmanifest_path(self.post_id)
            .ok_or_else(|| Error::Runtime("no app data dir".to_string()))?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_string_pretty(self)?)?;
        fs::rename(&tmp, &path)?;
        Ok(())
    }

    /// Delete the manifest (uninstall).
    pub fn remove(post_id: i64) -> Result<(), Error> {
        if let Some(path) = paths::appmanifest_path(post_id) {
            if path.exists() {
                fs::remove_file(path)?;
            }
        }
        Ok(())
    }
}

/// Absolute install folder: `<library_root>/common/<installdir>`.
pub fn common_dir(library_root: &Path) -> PathBuf {
    library_root.join("common")
}

/// Absolute install path for a game: `<library>/common/<installdir>`.
pub fn game_install_dir(library_root: &Path, manifest: &AppManifest) -> PathBuf {
    common_dir(library_root).join(&manifest.installdir)
}

/// Staging folder for an in-progress download.
pub fn downloading_dir(library_root: &Path, post_id: i64) -> PathBuf {
    library_root.join("downloading").join(post_id.to_string())
}

/// Resolve the effective library root (ADR-0005): the `library-root` setting
/// override when present, else the default `<data_root>/library`.
/// An override dir is created on first resolution.
pub fn resolved_library_root(config_path: Option<&Path>) -> Result<PathBuf, Error> {
    if let Some(cfg) = config_path {
        let s = Settings::load(cfg)?;
        if let Some(root) = s.library_root.as_deref().filter(|r| !r.trim().is_empty()) {
            let p = PathBuf::from(root);
            fs::create_dir_all(&p)?;
            return Ok(p);
        }
    }
    paths::library_dir().ok_or_else(|| Error::Runtime("no app data dir".to_string()))
}

/// Path of `libraryfolders.json` under the resolved library root.
pub fn resolved_libraryfolders_path(config_path: Option<&Path>) -> Result<PathBuf, Error> {
    Ok(resolved_library_root(config_path)?.join("libraryfolders.json"))
}

/// Path of an app manifest under the resolved library root.
pub fn resolved_appmanifest_path(
    config_path: Option<&Path>,
    post_id: i64,
) -> Result<PathBuf, Error> {
    Ok(resolved_library_root(config_path)?.join(format!("appmanifest_{post_id}.json")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("lz_lib_test_{}_{}", std::process::id(), nanos))
    }

    #[test]
    fn folders_loads_empty_then_assigns_game() {
        let dir = unique_dir();
        let path = dir.join("libraryfolders.json");
        let mut folders = LibraryFolders::load(&path).expect("missing file => default");
        assert!(folders.folders.is_empty());

        let root = folders.ensure_default().expect("default root");
        assert!(root.is_absolute());
        assert_eq!(folders.folders.len(), 1);

        folders.assign_game(1234, 42);
        folders.save(&path).expect("saves");

        let reloaded = LibraryFolders::load(&path).expect("reloads");
        let games: Vec<(i64, u64)> = reloaded
            .iter_games()
            .map(|(_, id, size)| (id, size))
            .collect();
        assert_eq!(games, vec![(1234, 42)]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn appmanifest_roundtrips_atomically() {
        let dir = unique_dir();
        let m = AppManifest {
            post_id: 999,
            title: "Treasure of Nadia".to_string(),
            installdir: "Treasure of Nadia".to_string(),
            version_label: Some("1.0".to_string()),
            platform: Some("PC".to_string()),
            state_flags: 4,
            size_on_disk: 1024,
            ..Default::default()
        };
        let written_path = paths::appmanifest_path(m.post_id).expect("real path");
        if let Some(parent) = written_path.parent() {
            fs::create_dir_all(parent).expect("mkdir");
        }
        m.write().expect("writes through paths::appmanifest_path");

        let loaded = AppManifest::load_from(&written_path).expect("loads");
        let m = loaded.expect("exists");
        assert_eq!(m.title, "Treasure of Nadia");
        assert_eq!(m.version_label.as_deref(), Some("1.0"));
        AppManifest::remove(999).expect("cleanup manifest");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn manifest_missing_is_not_installed() {
        let dir = unique_dir();
        let path = dir.join("appmanifest_42.json");
        let loaded = AppManifest::load_from(&path).expect("loads");
        assert!(loaded.is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn install_dirs_follow_steam_shape() {
        let root = PathBuf::from("X:/stor/library");
        let m = AppManifest {
            installdir: "My Game".to_string(),
            ..Default::default()
        };
        assert_eq!(common_dir(&root), PathBuf::from("X:/stor/library/common"));
        assert_eq!(
            game_install_dir(&root, &m),
            PathBuf::from("X:/stor/library/common/My Game")
        );
        assert_eq!(
            downloading_dir(&root, 7),
            PathBuf::from("X:/stor/library/downloading/7")
        );
    }

    #[test]
    fn resolved_root_uses_library_root_setting() {
        let dir = unique_dir();
        let cfg = dir.join("config.json");
        let settings = Settings {
            library_root: Some("D:/MyLewdzoneLibrary".to_string()),
            ..Default::default()
        };
        settings.save(&cfg).expect("settings saved");

        let root = resolved_library_root(Some(&cfg)).expect("resolves override");
        assert_eq!(root, PathBuf::from("D:/MyLewdzoneLibrary"));

        assert_eq!(
            resolved_libraryfolders_path(Some(&cfg)).expect("folders path"),
            root.join("libraryfolders.json")
        );
        assert_eq!(
            resolved_appmanifest_path(Some(&cfg), 12).expect("manifest path"),
            root.join("appmanifest_12.json")
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolved_root_falls_back_to_default_library_dir() {
        assert!(resolved_library_root(None)
            .expect("default root")
            .is_absolute());
        assert!(resolved_libraryfolders_path(None)
            .expect("folders path")
            .ends_with("libraryfolders.json"));
    }
}
