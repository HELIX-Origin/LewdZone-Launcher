//! uTorrent / BitTorrent adapter — cross-platform torrent client,
//! handles magnet links and local `.torrent` files.

use std::path::{Path, PathBuf};

use super::{find_in_path, first_existing, spawn_detached, DownloadManager, Kind};
use crate::core::Error;

/// Executable basenames searched on `$PATH`.
const PATH_NAMES: &[&str] = &[
    "uTorrent.exe",
    "ut.exe",
    "bittorrent.exe",
    "utorrent",
    "ut",
    "bittorrent",
];

/// Well-known install directories (Windows + macOS apps may vary; PATH wins).
fn candidates() -> Vec<PathBuf> {
    let mut v = Vec::new();
    let pf = std::env::var_os("ProgramFiles").map(PathBuf::from);
    if let Some(pf) = pf {
        v.push(pf.join("uTorrent").join("uTorrent.exe"));
        v.push(pf.join("BitTorrent").join("BitTorrent.exe"));
        v.push(pf.join("qBittorrent").join("qbittorrent.exe"));
    }
    let home = std::env::var_os("HOME").map(PathBuf::from);
    if let Some(home) = home {
        v.push(home.join("Applications").join("uTorrent.app"));
        v.push(home.join(".local/share/applications").join("uTorrent.app"));
    }
    v
}

#[derive(Debug, Default)]
pub struct Torrent;

impl DownloadManager for Torrent {
    fn name(&self) -> &'static str {
        "utorrent"
    }

    fn platforms(&self) -> &'static [&'static str] {
        &["windows", "linux", "macos"]
    }

    fn handles_kind(&self) -> Kind {
        Kind::Torrent
    }

    fn detect(&self) -> Option<PathBuf> {
        // PATH first (docs: override → PATH → well-known install dirs).
        find_in_path(PATH_NAMES).or_else(|| first_existing(&candidates()))
    }

    fn launch(&self, url: &str, _target_dir: &Path, _filename: Option<&str>) -> Result<(), Error> {
        let exe = self.detect().ok_or_else(|| {
            Error::DmMissing(
                "no torrent client detected; install uTorrent, BitTorrent, or qBittorrent".into(),
            )
        })?;
        // Positional magnet/`.torrent` argument; URL is data, never a shell string.
        spawn_detached(&exe, &[url])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn torrent_identity_and_contract() {
        let m = Torrent;
        assert_eq!(m.name(), "utorrent");
        assert!(m.platforms().contains(&"windows"));
        assert!(m.platforms().contains(&"linux"));
        assert!(m.platforms().contains(&"macos"));
        assert_eq!(m.handles_kind(), Kind::Torrent);
    }

    #[test]
    fn torrent_launch_without_install_is_missing_manager() {
        if Torrent.detect().is_some() {
            return;
        }
        let err = Torrent
            .launch("magnet:?xt=urn:btih:abc", Path::new("."), None)
            .unwrap_err();
        assert!(matches!(err, Error::DmMissing(_)));
    }
}
