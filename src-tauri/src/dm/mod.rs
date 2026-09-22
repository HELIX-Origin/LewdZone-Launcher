//! `dm` — download-manager adapter layer (ADR-0002, Rule 07).
//!
//! Every adapter implements the [`DownloadManager`] trait and registers in a
//! per-platform [`registry`]. The active manager comes from the `dm` setting
//! key; detection follows the dm-detector table (override → well-known dirs →
//! PATH). Spawning is always silent + detached via argv arrays (never a shell).

pub mod folder;
pub mod torrent;

#[cfg(windows)]
pub mod fdm;
#[cfg(windows)]
pub mod idm;

use std::path::{Path, PathBuf};

use crate::core::Error;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// What kind of payload an adapter can accept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Direct HTTP(S) file URLs.
    Http,
    /// Magnet links / `.torrent` URLs.
    Torrent,
}

/// A pluggable download manager (FDM, IDM, uTorrent/BitTorrent, …).
pub trait DownloadManager {
    /// Canonical adapter id, e.g. `fdm`, `idm`, `utorrent`.
    fn name(&self) -> &'static str;
    /// Host platforms this build supports (`windows`, `linux`, `macos`).
    fn platforms(&self) -> &'static [&'static str];
    /// Which payload kind this manager accepts.
    fn handles_kind(&self) -> Kind;
    /// Locate the binary, or `None` when not installed (dm-detector table).
    fn detect(&self) -> Option<PathBuf>;
    /// Spawn a silent, detached download for `url`; errors only on spawn
    /// failure, never while the manager runs. `target_dir` is a hint for
    /// managers that can place files directly (IDM).
    fn launch(&self, url: &str, target_dir: &Path, filename: Option<&str>) -> Result<(), Error>;
}

/// Build the registry of managers present on the current platform.
pub fn registry() -> Vec<Box<dyn DownloadManager>> {
    let mut all: Vec<Box<dyn DownloadManager>> = Vec::new();
    #[cfg(windows)]
    {
        all.push(Box::new(crate::dm::fdm::Fdm));
        all.push(Box::new(crate::dm::idm::Idm));
    }
    all.push(Box::new(torrent::Torrent));
    all.retain(|m| m.platforms().contains(&std::env::consts::OS));
    all
}

/// The adapter with id `name`, if the platform supports it.
pub fn find(name: &str) -> Option<Box<dyn DownloadManager>> {
    registry()
        .into_iter()
        .find(|m| m.name() == name)
        .map(|m| m as Box<dyn DownloadManager>)
}

/// Ordered list of every adapter id the current platform can host.
pub fn available_names() -> Vec<&'static str> {
    registry().into_iter().map(|m| m.name()).collect()
}

/// Spawn `cmd && args` silently and detach (never wait).
fn spawn_detached(cmd: &Path, args: &[&str]) -> Result<(), Error> {
    let mut proc = std::process::Command::new(cmd);
    proc.args(args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        proc.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        proc.process_group(0);
    }
    proc.spawn()
        .map(|_| ())
        .map_err(|e| Error::Runtime(format!("failed to spawn '{}': {e}", cmd.display())))
}

/// Check `candidates` for an existing executable; return the first hit.
pub fn first_existing(candidates: &[PathBuf]) -> Option<PathBuf> {
    candidates.iter().find(|p| p.exists()).cloned()
}

/// Probe `$PATH` for any of `names`, returning the first executable found.
pub fn find_in_path(names: &[&str]) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        for name in names {
            let cand = dir.join(name);
            if cand.is_file() {
                return Some(cand);
            }
        }
    }
    None
}

/// Resolve the download root: `download-root` setting or library dir default.
pub fn download_root(ctx: &crate::core::Context) -> Result<PathBuf, Error> {
    use crate::core::settings::Settings;
    let s = Settings::load(&ctx.config_path)?;
    if let Some(root) = s
        .get_value("download-root")
        .and_then(|v| v.as_str().map(str::to_string))
    {
        if !root.is_empty() {
            return Ok(PathBuf::from(root));
        }
    }
    crate::core::paths::library_dir()
        .ok_or_else(|| Error::Runtime("cannot resolve download root (no app data dir)".to_string()))
}

/// Read the `dm` setting (or validate a requested name) as the active manager.
pub fn active_name(ctx: &crate::core::Context) -> Result<String, Error> {
    use crate::core::settings::Settings;
    let s = Settings::load(&ctx.config_path)?;
    Ok(s.get_value("dm")
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_else(|| "fdm".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_only_contains_platform_managers() {
        for m in registry() {
            assert!(m.platforms().contains(&std::env::consts::OS));
        }
    }

    #[test]
    fn available_names_are_nonempty_and_unique() {
        let names = available_names();
        assert!(!names.is_empty());
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(names.len(), sorted.len(), "duplicate adapter ids");
    }

    #[test]
    fn first_existing_skips_missing_files() {
        let tmp = std::env::temp_dir();
        let missing = tmp.join("definitely-missing-dm-tool.exe");
        let present = tmp;
        assert_eq!(
            first_existing(&[missing.clone(), present.clone()]),
            Some(present)
        );
        assert!(first_existing(&[missing]).is_none());
    }
}
