//! FDM adapter — Windows-only silent HTTP downloader (`fdm.exe -fs <url>`).

use std::path::{Path, PathBuf};

use super::{first_existing, spawn_detached, DownloadManager, Kind};
use crate::core::Error;

/// Candidate install locations, in detection order.
fn candidates() -> Vec<PathBuf> {
    let mut v = Vec::new();
    let pf = std::env::var_os("ProgramFiles").map(PathBuf::from);
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        v.push(
            PathBuf::from(local)
                .join("Programs")
                .join("Free Download Manager")
                .join("fdm.exe"),
        );
    }
    if let Some(pf) = pf {
        v.push(pf.join("Free Download Manager").join("fdm.exe"));
        v.push(pf.join("FDM").join("fdm.exe"));
    }
    let pf86 = std::env::var_os("ProgramFiles(x86)").map(PathBuf::from);
    if let Some(pf86) = pf86 {
        v.push(pf86.join("Free Download Manager").join("fdm.exe"));
    }
    v
}

#[derive(Debug, Default)]
pub struct Fdm;

impl DownloadManager for Fdm {
    fn name(&self) -> &'static str {
        "fdm"
    }

    fn platforms(&self) -> &'static [&'static str] {
        &["windows"]
    }

    fn handles_kind(&self) -> Kind {
        Kind::Http
    }

    fn detect(&self) -> Option<PathBuf> {
        first_existing(&candidates())
    }

    fn launch(&self, url: &str, _target_dir: &Path, _filename: Option<&str>) -> Result<(), Error> {
        let exe = self.detect().ok_or_else(|| {
            Error::DmMissing(
                "FDM not detected; set the configured path or install Free Download Manager".into(),
            )
        })?;
        spawn_detached(&exe, &["-fs", url])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fdm_identity_and_contract() {
        let m = Fdm;
        assert_eq!(m.name(), "fdm");
        assert_eq!(m.platforms(), &["windows"]);
        assert_eq!(m.handles_kind(), Kind::Http);
    }

    #[test]
    fn fdm_candidates_are_specific_paths() {
        let c = candidates();
        assert!(!c.is_empty());
        for p in &c {
            assert!(p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .contains("fdm.exe"));
        }
    }

    #[test]
    fn fdm_launch_without_install_is_missing_manager() {
        if Fdm.detect().is_some() {
            return; // real install present; nothing to assert offline
        }
        let err = Fdm
            .launch("https://example.com/file.zip", Path::new("."), None)
            .unwrap_err();
        assert!(matches!(err, Error::DmMissing(_)));
    }
}
