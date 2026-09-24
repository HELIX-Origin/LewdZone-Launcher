//! Library folder registry + per-game manifests (ADR-0005).
//! `libraryfolders.json` holds the ordered roots;
//! `appmanifest_<post_id>.json` is the per-game record.
//!
//! Installed apps are discovered from `<lzapps_root>/<slug>/app.json` manifests
//! written by the extractor (see `core::extract`).

use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::paths;
use crate::core::settings::Settings;
use crate::core::{Context, Error};

const DEFAULT_LIBRARY_LABEL: &str = "Default";

/// Ordered library roots: `{ index: LibraryFolder }` (order analog).
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(transparent)]
pub struct LibraryFolders {
    pub folders: BTreeMap<u32, LibraryFolder>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct LibraryFolder {
    pub path: String,
    pub label: Option<String>,
    /// `games: { post_id: size_on_disk }` (per-root installed games).
    #[serde(default)]
    pub games: BTreeMap<i64, u64>,
}

/// `appmanifest_<post_id>.json` — the per-game record.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppManifest {
    pub post_id: i64,
    pub title: String,
    /// Install folder name under `common/` (installdir analog).
    pub installdir: String,
    pub version_label: Option<String>,
    pub platform: Option<String>,
    pub tab: Option<String>,
    pub host_slug: Option<String>,
    /// State flags analog: 4 = fully installed.
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

/// A discovered installed app, read from `<lzapps_root>/<slug>/app.json`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct InstalledApp {
    pub slug: String,
    pub post_id: Option<i64>,
    pub title: String,
    pub version: String,
    pub platform: String,
    pub tab: String,
    pub engine: Option<String>,
    pub install_path: PathBuf,
    pub candidates: Vec<String>,
    pub launch_exe: String,
    pub installed_at: Option<String>,
    pub size_on_disk: u64,
}

/// Result summary of scanning an extracted games directory.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScanReport {
    pub scanned_dir: String,
    pub found_count: usize,
    pub queued_count: usize,
    pub added_games: Vec<String>,
}

/// List every installed app under the effective `installed/` or `lzapps/` root.
pub fn list_installed(ctx: &Context) -> Result<Vec<InstalledApp>, Error> {
    let mut roots = Vec::new();
    if let Ok(inst_root) = crate::core::folder::installed_root(ctx) {
        if inst_root.exists() && !roots.contains(&inst_root) {
            roots.push(inst_root);
        }
    }
    if let Ok(lz_root) = crate::core::folder::lzapps_root(ctx) {
        if lz_root.exists() && !roots.contains(&lz_root) {
            roots.push(lz_root);
        }
    }

    let mut apps = Vec::new();
    let mut seen_slugs = std::collections::HashSet::new();

    for root in roots {
        let Ok(entries) = fs::read_dir(&root) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            // Case 1: direct root/<slug>/app.json
            let manifest_path = path.join("app.json");
            if manifest_path.is_file() {
                if let Ok(raw) = fs::read_to_string(&manifest_path) {
                    if let Ok(manifest) =
                        serde_json::from_str::<crate::core::extract::AppJson>(&raw)
                    {
                        if seen_slugs.insert(manifest.slug.clone()) {
                            let install_path = if !manifest.install_path.trim().is_empty()
                                && Path::new(&manifest.install_path).exists()
                            {
                                PathBuf::from(&manifest.install_path)
                            } else {
                                path.clone()
                            };

                            apps.push(InstalledApp {
                                slug: manifest.slug,
                                post_id: manifest.post_id,
                                title: manifest.title,
                                version: manifest.version,
                                platform: manifest.platform,
                                tab: manifest.tab,
                                engine: manifest.engine,
                                install_path: install_path.clone(),
                                candidates: manifest.candidates,
                                launch_exe: manifest.launch_exe,
                                installed_at: Some(manifest.installed_at),
                                size_on_disk: dir_size(&install_path).unwrap_or(0),
                            });
                        }
                    }
                }
                continue;
            }

            // Case 2: engine folder root/<engine>/<slug>/app.json
            if let Ok(sub_entries) = fs::read_dir(&path) {
                for sub_entry in sub_entries.flatten() {
                    let sub_path = sub_entry.path();
                    if !sub_path.is_dir() {
                        continue;
                    }
                    let sub_manifest_path = sub_path.join("app.json");
                    if sub_manifest_path.is_file() {
                        if let Ok(raw) = fs::read_to_string(&sub_manifest_path) {
                            if let Ok(manifest) =
                                serde_json::from_str::<crate::core::extract::AppJson>(&raw)
                            {
                                if seen_slugs.insert(manifest.slug.clone()) {
                                    let install_path = if !manifest.install_path.trim().is_empty()
                                        && Path::new(&manifest.install_path).exists()
                                    {
                                        PathBuf::from(&manifest.install_path)
                                    } else {
                                        sub_path.clone()
                                    };

                                    apps.push(InstalledApp {
                                        slug: manifest.slug,
                                        post_id: manifest.post_id,
                                        title: manifest.title,
                                        version: manifest.version,
                                        platform: manifest.platform,
                                        tab: manifest.tab,
                                        engine: manifest.engine,
                                        install_path: install_path.clone(),
                                        candidates: manifest.candidates,
                                        launch_exe: manifest.launch_exe,
                                        installed_at: Some(manifest.installed_at),
                                        size_on_disk: dir_size(&install_path).unwrap_or(0),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    apps.sort_by_key(|a| a.title.to_lowercase());
    Ok(apps)
}

/// Clean folder name to isolate canonical game title.
fn clean_folder_title(raw: &str) -> String {
    let mut cleaned = raw.to_string();
    while let Some(start) = cleaned.find('[') {
        if let Some(end) = cleaned[start..].find(']') {
            cleaned.replace_range(start..start + end + 1, " ");
        } else {
            break;
        }
    }
    while let Some(start) = cleaned.find('(') {
        if let Some(end) = cleaned[start..].find(')') {
            cleaned.replace_range(start..start + end + 1, " ");
        } else {
            break;
        }
    }

    // Replace underscores with spaces so "Treasure_of_Nadia" -> "Treasure of Nadia"
    cleaned = cleaned.replace('_', " ");

    let lower = cleaned.to_lowercase();
    if let Some(idx) = lower.find(" - version") {
        cleaned.truncate(idx);
    } else if let Some(idx) = lower.find(" version ") {
        cleaned.truncate(idx);
    } else if let Some(idx) = lower.find(" - v") {
        cleaned.truncate(idx);
    } else if let Some(idx) = lower.find(" v") {
        let rest = &lower[idx + 2..];
        if rest.starts_with(|c: char| c.is_ascii_digit() || c == '.') {
            cleaned.truncate(idx);
        }
    }

    // Strip trailing platform markers if present
    let lower_trim = cleaned.trim().to_lowercase();
    for suffix in &[" - pc", " - mac", " - linux", " pc", " mac", " linux"] {
        if lower_trim.ends_with(suffix) {
            let new_len = cleaned.trim().len() - suffix.len();
            cleaned = cleaned.trim()[..new_len].to_string();
            break;
        }
    }

    cleaned
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

/// Check if an .exe file contains an embedded SFX archive (7z, RAR, or Zip).
pub fn is_sfx_archive(path: &Path) -> bool {
    let Ok(mut f) = fs::File::open(path) else {
        return false;
    };
    let mut buf = vec![0u8; 65536];
    let n = match f.read(&mut buf) {
        Ok(n) => n,
        Err(_) => return false,
    };
    let slice = &buf[..n];

    let has_7z = slice.windows(6).any(|w| w == b"7z\xBC\xAF\x27\x1C");
    let has_rar = slice
        .windows(7)
        .any(|w| w == b"Rar!\x1A\x07\x00" || w == b"Rar!\x1A\x07\x01");
    let has_zip = slice.windows(4).any(|w| w == b"PK\x03\x04");

    has_7z || has_rar || has_zip
}

/// Supported archive extensions: zip, 7z, rar, tar, tar.gz, tgz, tar.bz2, tbz2, tar.xz, txz, and sfx .exe.
pub fn is_supported_archive(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    name.ends_with(".zip")
        || name.ends_with(".7z")
        || name.ends_with(".rar")
        || name.ends_with(".tar")
        || name.ends_with(".tar.gz")
        || name.ends_with(".tgz")
        || name.ends_with(".tar.bz2")
        || name.ends_with(".tbz2")
        || name.ends_with(".tar.xz")
        || name.ends_with(".txz")
        || (name.ends_with(".exe") && is_sfx_archive(path))
}

fn clean_archive_stem(file_name: &str) -> String {
    let mut s = file_name.to_string();
    let lower = s.to_lowercase();
    for ext in &[
        ".tar.gz", ".tar.bz2", ".tar.xz", ".tgz", ".tbz2", ".txz", ".zip", ".7z", ".rar", ".exe",
    ] {
        if lower.ends_with(ext) {
            s.truncate(s.len() - ext.len());
            break;
        }
    }
    s
}

/// Extract clean title, version, and platform from an archive's filename.
fn parse_archive_filename(file_name: &str) -> (String, String, String) {
    let stem = clean_archive_stem(file_name);
    let clean_title = clean_folder_title(&stem);

    let lower = stem.to_lowercase();
    let version = if let Some(idx) = lower.find("version ") {
        let rest = &stem[idx + "version ".len()..];
        let ver = rest.split([' ', '-', '_']).next().unwrap_or("latest");
        ver.trim().to_string()
    } else if let Some(idx) = lower.find("- v") {
        let rest = &stem[idx + 3..];
        let ver = rest.split([' ', '-', '_']).next().unwrap_or("latest");
        ver.trim().to_string()
    } else if let Some(idx) = lower.find("_v") {
        let rest = &stem[idx + 2..];
        let ver = rest.split([' ', '-', '_']).next().unwrap_or("latest");
        ver.trim().to_string()
    } else if let Some(idx) = lower.find(" v") {
        let rest = &stem[idx + 2..];
        if rest.starts_with(|c: char| c.is_ascii_digit() || c == '.') {
            let ver = rest.split([' ', '-', '_']).next().unwrap_or("latest");
            ver.trim().to_string()
        } else {
            "latest".to_string()
        }
    } else {
        "latest".to_string()
    };

    let platform = if lower.contains("- pc") || lower.ends_with(" pc") || lower.contains("_pc") {
        "pc".to_string()
    } else if lower.contains("- mac") || lower.ends_with(" mac") || lower.contains("_mac") {
        "mac".to_string()
    } else if lower.contains("- linux") || lower.ends_with(" linux") || lower.contains("_linux") {
        "linux".to_string()
    } else if cfg!(target_os = "windows") {
        "pc".to_string()
    } else if cfg!(target_os = "macos") {
        "mac".to_string()
    } else {
        "pc".to_string()
    };

    (clean_title, version, platform)
}

/// Find game executable candidates in an extracted folder.
fn find_candidates(dir: &Path) -> Vec<String> {
    let mut candidates = Vec::new();
    let mut search_dirs = vec![dir.to_path_buf()];

    if let Ok(entries) = fs::read_dir(dir) {
        let mut subdirs = Vec::new();
        for e in entries.flatten() {
            if let Ok(m) = e.metadata() {
                if m.is_dir() {
                    subdirs.push(e.path());
                }
            }
        }
        if subdirs.len() == 1 {
            search_dirs.push(subdirs.remove(0));
        }
    }

    for d in search_dirs {
        let Ok(entries) = fs::read_dir(&d) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if ext_lower == "exe"
                        || ext_lower == "app"
                        || ext_lower == "x86_64"
                        || ext_lower == "sh"
                    {
                        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        let name_lower = name.to_lowercase();
                        if !name_lower.contains("unins")
                            && !name_lower.contains("crash")
                            && !name_lower.contains("reporter")
                            && !name_lower.contains("unitycrash")
                            && !name_lower.contains("dxsetup")
                            && !name_lower.contains("vcredist")
                        {
                            let rel = path.strip_prefix(dir).unwrap_or(&path);
                            candidates.push(rel.to_string_lossy().replace('\\', "/"));
                        }
                    }
                }
            }
        }
    }

    candidates.sort_by_key(|c| {
        let lower = c.to_lowercase();
        if lower.ends_with("game.exe") || lower.ends_with("nw.exe") {
            0
        } else if lower.ends_with(".exe") {
            1
        } else {
            2
        }
    });

    candidates
}

/// Detect engine from folder layout.
fn detect_engine(dir: &Path, candidates: &[String]) -> Option<String> {
    for c in candidates {
        let lower = c.to_lowercase();
        if lower.ends_with("nw.exe") {
            return Some("HTML/NW.js".to_string());
        }
    }
    if dir.join("game").is_dir() || dir.join("renpy").is_dir() {
        return Some("Ren'Py".to_string());
    }
    if dir.join("package.json").exists() || dir.join("www").is_dir() {
        return Some("HTML/NW.js".to_string());
    }
    if dir.join("UnityPlayer.dll").exists() || dir.join("UnityCrashHandler64.exe").exists() {
        return Some("Unity".to_string());
    }
    if dir.join("Game.rgss3a").exists() || dir.join("Game.ini").exists() {
        return Some("RPGM".to_string());
    }
    None
}

/// Detect engine folder name from archive or directory path (e.g. downloads/renpy/... or installed/renpy/...).
pub fn detect_engine_from_path(path: &Path) -> Option<String> {
    let mut cur = path.parent();
    while let Some(p) = cur {
        if let Some(parent) = p.parent() {
            if let Some(parent_name) = parent.file_name().and_then(|n| n.to_str()) {
                if parent_name.eq_ignore_ascii_case("downloads")
                    || parent_name.eq_ignore_ascii_case("installed")
                {
                    if let Some(engine_name) = p.file_name().and_then(|n| n.to_str()) {
                        return Some(crate::core::folder::engine_to_folder(Some(engine_name)));
                    }
                }
            }
        }
        cur = p.parent();
    }
    None
}

/// Scan a directory where the user stores download zips or extracts games, and
/// register/enqueue matching games into `lzapps/<slug>/app.json` and the progress queue.
pub fn scan_games_dir(
    ctx: &Context,
    path_override: Option<&Path>,
    queue: Option<&crate::core::queue::Queue>,
) -> Result<ScanReport, Error> {
    let settings = Settings::load(&ctx.config_path)?;
    let scan_dir = if let Some(p) = path_override {
        p.to_path_buf()
    } else if let Some(dir) = settings.games_dir.as_ref().filter(|s| !s.trim().is_empty()) {
        PathBuf::from(dir)
    } else if let Some(root) = settings
        .library_root
        .as_ref()
        .filter(|s| !s.trim().is_empty())
    {
        let p = PathBuf::from(root);
        if p.join("Games").exists() {
            p.join("Games")
        } else {
            p
        }
    } else {
        return Err(Error::Usage(
            "no extracted games directory configured; please set Extracted Games Directory in Settings"
                .to_string(),
        ));
    };

    if !scan_dir.exists() {
        return Err(Error::Usage(format!(
            "scan directory does not exist: {}",
            scan_dir.display()
        )));
    }

    // Auto-create downloads and installed folders with all engine subfolders
    let _ = crate::core::folder::initialize_library_structure(&scan_dir);

    let mut archive_files = Vec::new();
    let mut game_dirs = Vec::new();

    let mut stack = vec![(scan_dir.clone(), 0usize)];
    while let Some((dir, depth)) = stack.pop() {
        if depth > 4 {
            continue;
        }
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.starts_with('.') || name == "_parts" {
                continue;
            }
            if path.is_file() {
                if is_supported_archive(&path) {
                    archive_files.push(path);
                }
            } else if path.is_dir() {
                // If it's an installed or lzapps folder, inspect each child installed game folder
                if name.eq_ignore_ascii_case("installed") || name.eq_ignore_ascii_case("lzapps") {
                    if let Ok(sub_entries) = fs::read_dir(&path) {
                        for sub_entry in sub_entries.flatten() {
                            let sub_path = sub_entry.path();
                            if !sub_path.is_dir() {
                                continue;
                            }
                            if sub_path.join("app.json").exists()
                                || !find_candidates(&sub_path).is_empty()
                            {
                                game_dirs.push(sub_path);
                            } else {
                                // Subfolder could be an engine folder (e.g. installed/renpy/)
                                if let Ok(game_entries) = fs::read_dir(&sub_path) {
                                    for game_entry in game_entries.flatten() {
                                        let gp = game_entry.path();
                                        if gp.is_dir()
                                            && (gp.join("app.json").exists()
                                                || !find_candidates(&gp).is_empty())
                                        {
                                            game_dirs.push(gp);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    continue;
                }
                // Check if directory itself contains game executables
                let candidates = find_candidates(&path);
                if !candidates.is_empty() {
                    game_dirs.push(path.clone());
                }
                // Recurse into subdirectories (e.g. downloads, Games) to search for archives
                stack.push((path, depth + 1));
            }
        }
    }

    let lzapps_root = crate::core::folder::lzapps_root(ctx)?;
    fs::create_dir_all(&lzapps_root)?;

    let conn = ctx.open_db().ok();
    let mut added_games = Vec::new();
    let mut queued_count = 0;

    // 1. Process archive files (zip, 7z, rar, tar, tar.gz, sfx .exe) for auto-extraction
    for archive_path in &archive_files {
        let file_name = archive_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if file_name.is_empty() {
            continue;
        }

        let (clean_title, version, platform) = parse_archive_filename(file_name);
        if clean_title.is_empty() {
            continue;
        }

        let mut matched_slug = None;
        let mut matched_title = clean_title.clone();
        let mut matched_engine = None;

        if let Some(ref db) = conn {
            let lower_title = clean_title.to_lowercase();
            let generated_slug = crate::core::folder::sanitize_segment(&clean_title)
                .to_lowercase()
                .replace(' ', "-");

            let query = "SELECT post_id, slug, title, engine FROM game
                         WHERE LOWER(title) = ?1 OR slug = ?2 OR LOWER(title) LIKE ?3
                         LIMIT 1";
            let like_term = format!("%{}%", lower_title);
            let mut stmt = db.prepare(query).ok();
            if let Some(ref mut st) = stmt {
                let row = st
                    .query_row(
                        rusqlite::params![lower_title, generated_slug, like_term],
                        |r| {
                            Ok((
                                r.get::<_, i64>(0)?,
                                r.get::<_, String>(1)?,
                                r.get::<_, String>(2)?,
                                r.get::<_, Option<String>>(3)?,
                            ))
                        },
                    )
                    .ok();
                if let Some((_pid, slug, title, eng)) = row {
                    matched_slug = Some(slug);
                    matched_title = title;
                    matched_engine = eng;
                }
            }
        }

        let slug = matched_slug.unwrap_or_else(|| {
            crate::core::folder::sanitize_segment(&clean_title)
                .to_lowercase()
                .replace(' ', "-")
        });

        // Determine engine folder from archive path (e.g. downloads/renpy/...) or DB
        let path_engine = detect_engine_from_path(archive_path);
        let engine_folder = path_engine
            .or_else(|| {
                matched_engine
                    .as_deref()
                    .map(|e| crate::core::folder::engine_to_folder(Some(e)))
            })
            .unwrap_or_else(|| "other".to_string());

        let target_base = if scan_dir.join("installed").is_dir() {
            scan_dir.join("installed")
        } else if let Ok(inst_root) = crate::core::folder::installed_root(ctx) {
            inst_root
        } else if scan_dir.join("lzapps").is_dir() {
            scan_dir.join("lzapps")
        } else if let Some(gdir) = settings.games_dir.as_ref().filter(|s| !s.trim().is_empty()) {
            PathBuf::from(gdir)
        } else {
            lzapps_root.clone()
        };

        let target_dir = target_base.join(&slug);

        let manifest_path = target_dir.join("app.json");
        let legacy_manifest = target_base
            .join(&engine_folder)
            .join(&slug)
            .join("app.json");
        let already_extracted = manifest_path.exists()
            || legacy_manifest.exists()
            || (target_dir.exists() && !find_candidates(&target_dir).is_empty());

        if !already_extracted {
            if let Some(q) = queue {
                let _ = q.enqueue_extract(
                    slug.clone(),
                    version,
                    platform,
                    archive_path.to_string_lossy().into_owned(),
                    target_dir.to_string_lossy().into_owned(),
                );
                queued_count += 1;
                added_games.push(format!("{matched_title} (queued extraction)"));
            }
        }
    }

    // 2. Process already-extracted directory entries
    for gdir in game_dirs {
        let folder_name = gdir.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if folder_name.is_empty() || folder_name.starts_with('.') || folder_name == "_parts" {
            continue;
        }

        let candidates = find_candidates(&gdir);
        if candidates.is_empty() {
            continue;
        }

        let clean_title = clean_folder_title(folder_name);
        if clean_title.is_empty() {
            continue;
        }

        let mut matched_slug = None;
        let mut matched_title = clean_title.clone();
        let mut matched_post_id = None;
        let mut matched_engine = detect_engine(&gdir, &candidates);

        if let Some(ref db) = conn {
            let lower_title = clean_title.to_lowercase();
            let generated_slug = crate::core::folder::sanitize_segment(&clean_title)
                .to_lowercase()
                .replace(' ', "-");

            let query = "SELECT post_id, slug, title, engine FROM game
                         WHERE LOWER(title) = ?1 OR slug = ?2 OR LOWER(title) LIKE ?3
                         LIMIT 1";
            let like_term = format!("%{}%", lower_title);
            let mut stmt = db.prepare(query).ok();
            if let Some(ref mut st) = stmt {
                let row = st
                    .query_row(
                        rusqlite::params![lower_title, generated_slug, like_term],
                        |r| {
                            Ok((
                                r.get::<_, i64>(0)?,
                                r.get::<_, String>(1)?,
                                r.get::<_, String>(2)?,
                                r.get::<_, Option<String>>(3)?,
                            ))
                        },
                    )
                    .ok();
                if let Some((pid, slug, title, eng)) = row {
                    matched_post_id = Some(pid);
                    matched_slug = Some(slug);
                    matched_title = title;
                    if eng.is_some() {
                        matched_engine = eng;
                    }
                }
            }
        }

        let slug = matched_slug.unwrap_or_else(|| {
            crate::core::folder::sanitize_segment(&clean_title)
                .to_lowercase()
                .replace(' ', "-")
        });

        let launch_exe = candidates.first().cloned().unwrap_or_default();
        let app_json_path = gdir.join("app.json");

        let manifest = crate::core::extract::AppJson {
            format: "lzapp/v1".to_string(),
            slug: slug.clone(),
            post_id: matched_post_id,
            title: matched_title.clone(),
            version: "local".to_string(),
            platform: if cfg!(target_os = "windows") {
                "pc".to_string()
            } else if cfg!(target_os = "macos") {
                "mac".to_string()
            } else {
                "linux".to_string()
            },
            tab: "local".to_string(),
            engine: matched_engine,
            download_url: String::new(),
            install_path: gdir.to_string_lossy().into_owned(),
            installed_at: "local".to_string(),
            candidates,
            launch_exe,
        };

        let json = serde_json::to_string_pretty(&manifest)?;
        fs::write(&app_json_path, &json)?;

        // Also copy to lzapps_root/<slug>/app.json if lzapps_root exists and differs from gdir
        let lz_app_dir = lzapps_root.join(&slug);
        if lz_app_dir != gdir {
            let _ = fs::create_dir_all(&lz_app_dir);
            let _ = fs::write(lz_app_dir.join("app.json"), &json);
        }

        added_games.push(matched_title);
    }

    let found_count = added_games.len();
    Ok(ScanReport {
        scanned_dir: scan_dir.to_string_lossy().into_owned(),
        found_count,
        queued_count,
        added_games,
    })
}

fn dir_size(path: &Path) -> Result<u64, Error> {
    let mut total = 0u64;
    let mut stack = vec![path.to_path_buf()];
    while let Some(cur) = stack.pop() {
        for entry in fs::read_dir(&cur)? {
            let entry = entry?;
            let meta = entry.metadata()?;
            if meta.is_dir() {
                stack.push(entry.path());
            } else {
                total += meta.len();
            }
        }
    }
    Ok(total)
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
    fn install_dirs_follow_mirrored_shape() {
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
            library_root: Some("test-library-root".to_string()),
            ..Default::default()
        };
        settings.save(&cfg).expect("settings saved");

        let root = resolved_library_root(Some(&cfg)).expect("resolves override");
        assert_eq!(root, PathBuf::from("test-library-root"));

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

    #[test]
    fn list_installed_reads_lzapps_app_json() {
        let dir = unique_dir();
        fs::create_dir_all(&dir).unwrap();
        let cfg = dir.join("config.json");
        let db = dir.join("test.db");
        let lib_root = dir.to_string_lossy();
        let cfg_json = serde_json::json!({"library_root": lib_root.to_string()});
        fs::write(&cfg, serde_json::to_string(&cfg_json).unwrap()).unwrap();
        let ctx = Context::new(db, cfg);

        let lzapps = dir.join("lzapps");
        let install = lzapps.join("treasure-of-nadia");
        fs::create_dir_all(&install).unwrap();
        fs::write(install.join("game.exe"), b"fake exe").unwrap();
        fs::write(
            install.join("app.json"),
            br#"{
                "format": "lewdzone-lzapp",
                "slug": "treasure-of-nadia",
                "post_id": 123,
                "title": "Treasure of Nadia",
                "version": "1.0117",
                "platform": "pc",
                "tab": "fileknot",
                "engine": "Ren'Py",
                "download_url": "https://fileknot.io/dl/x",
                "install_path": "treasure-of-nadia",
                "installed_at": "2026-01-01T00:00:00Z",
                "candidates": ["game.exe"],
                "launch_exe": ""
            }"#,
        )
        .unwrap();

        let apps = list_installed(&ctx).expect("lists installed apps");
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].slug, "treasure-of-nadia");
        assert_eq!(apps[0].title, "Treasure of Nadia");
        assert_eq!(apps[0].post_id, Some(123));
        assert!(apps[0].size_on_disk >= 8); // "fake exe" plus app.json

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parses_various_archive_filename_patterns() {
        let (title, ver, plat) =
            parse_archive_filename("Harem Hotel [Ongoing] - Version 0.19.1 - Fileknot - pc.zip");
        assert_eq!(title, "Harem Hotel");
        assert_eq!(ver, "0.19.1");
        assert_eq!(plat, "pc");

        let (title2, ver2, plat2) = parse_archive_filename("Treasure_of_Nadia_v1.0117_mac.tar.gz");
        assert_eq!(title2, "Treasure of Nadia");
        assert_eq!(ver2, "1.0117");
        assert_eq!(plat2, "mac");

        let (title3, ver3, plat3) =
            parse_archive_filename("Summertime Saga - Version 0.20.16 - linux.7z");
        assert_eq!(title3, "Summertime Saga");
        assert_eq!(ver3, "0.20.16");
        assert_eq!(plat3, "linux");
    }

    #[test]
    fn list_installed_reads_engine_subfolder_app_json() {
        let dir = unique_dir();
        fs::create_dir_all(&dir).unwrap();
        let cfg = dir.join("config.json");
        let db = dir.join("test.db");
        let lib_root = dir.to_string_lossy();
        let cfg_json = serde_json::json!({"library_root": lib_root.to_string()});
        fs::write(&cfg, serde_json::to_string(&cfg_json).unwrap()).unwrap();
        let ctx = Context::new(db, cfg);

        let installed_game = dir.join("installed").join("renpy").join("harem-hotel");
        fs::create_dir_all(&installed_game).unwrap();
        fs::write(installed_game.join("game.exe"), b"fake renpy exe").unwrap();
        fs::write(
            installed_game.join("app.json"),
            br#"{
                "format": "lzapp/v1",
                "slug": "harem-hotel",
                "post_id": 456,
                "title": "Harem Hotel",
                "version": "0.19.1",
                "platform": "pc",
                "tab": "fileknot",
                "engine": "Ren'Py",
                "download_url": "",
                "install_path": "",
                "installed_at": "2026-01-01T00:00:00Z",
                "candidates": ["game.exe"],
                "launch_exe": ""
            }"#,
        )
        .unwrap();

        let apps = list_installed(&ctx).expect("lists installed apps");
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].slug, "harem-hotel");
        assert_eq!(apps[0].title, "Harem Hotel");
        assert_eq!(apps[0].engine.as_deref(), Some("Ren'Py"));
        assert_eq!(apps[0].install_path, installed_game);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn detects_engine_from_downloads_path() {
        let p = Path::new(
            "G:/LewdZone/downloads/renpy/Harem Hotel [Ongoing] - Version 0.19.1/game.zip",
        );
        assert_eq!(detect_engine_from_path(p), Some("renpy".to_string()));

        let p2 = Path::new("G:/LewdZone/installed/unity/Wild Life/game.exe");
        assert_eq!(detect_engine_from_path(p2), Some("unity".to_string()));

        let p3 = Path::new("C:/Random/Folder/game.zip");
        assert_eq!(detect_engine_from_path(p3), None);
    }

    #[test]
    fn recognizes_supported_archive_extensions() {
        assert!(is_supported_archive(Path::new("game.zip")));
        assert!(is_supported_archive(Path::new("game.7z")));
        assert!(is_supported_archive(Path::new("game.rar")));
        assert!(is_supported_archive(Path::new("game.tar")));
        assert!(is_supported_archive(Path::new("game.tar.gz")));
        assert!(is_supported_archive(Path::new("game.tgz")));
        assert!(is_supported_archive(Path::new("game.tar.xz")));
        assert!(!is_supported_archive(Path::new("game.txt")));
    }
}
