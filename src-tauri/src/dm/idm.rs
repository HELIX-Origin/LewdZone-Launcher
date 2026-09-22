//! IDM adapter — Windows-only silent HTTP downloader
//! (`IDMan.exe /d <url> /n /p <target_dir>`).

use std::path::{Path, PathBuf};

use super::{first_existing, spawn_detached, DownloadManager, Kind};
use crate::core::Error;

/// Candidate install locations, in detection order.
fn candidates() -> Vec<PathBuf> {
    let mut v = Vec::new();
    let pf86 = std::env::var_os("ProgramFiles(x86)").map(PathBuf::from);
    if let Some(pf86) = pf86 {
        v.push(pf86.join("Internet Download Manager").join("IDMan.exe"));
    }
    let pf = std::env::var_os("ProgramFiles").map(PathBuf::from);
    if let Some(pf) = pf {
        v.push(pf.join("Internet Download Manager").join("IDMan.exe"));
    }
    let common = std::env::var_os("ProgramData").map(PathBuf::from);
    if let Some(common) = common {
        v.push(common.join("IDM").join("IDMan.exe"));
    }
    v
}

#[derive(Debug, Default)]
pub struct Idm;

impl DownloadManager for Idm {
    fn name(&self) -> &'static str {
        "idm"
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

    fn launch(&self, url: &str, target_dir: &Path, filename: Option<&str>) -> Result<(), Error> {
        let exe = self.detect().ok_or_else(|| {
            Error::DmMissing(
                "IDM not detected; set the configured path or install Internet Download Manager"
                    .into(),
            )
        })?;
        let mut args = vec!["/d", url, "/n", "/p"];
        let dir = target_dir.to_string_lossy().into_owned();
        args.push(&dir);
        if let Some(name) = filename {
            args.push("/f");
            args.push(name);
        }
        spawn_detached(&exe, &args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idm_identity_and_contract() {
        let m = Idm;
        assert_eq!(m.name(), "idm");
        assert_eq!(m.platforms(), &["windows"]);
        assert_eq!(m.handles_kind(), Kind::Http);
    }

    #[test]
    fn idm_candidates_are_specific_paths() {
        let c = candidates();
        assert!(!c.is_empty());
        for p in &c {
            assert!(p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .contains("IDMan.exe"));
        }
    }

    #[test]
    fn idm_launch_without_install_is_missing_manager() {
        if Idm.detect().is_some() {
            return;
        }
        let err = Idm
            .launch("https://example.com/file.zip", Path::new("."), None)
            .unwrap_err();
        assert!(matches!(err, Error::DmMissing(_)));
    }
}
