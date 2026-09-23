//! Extraction and installed-app manifest writer.
//!
//! After a direct-file download finishes, `.zip` archives are extracted into
//! `<lzapps_root>/<slug>/` and an itch.io-style `app.json` manifest is written
//! so the launcher can launch and manage the installed app. Updates replace
//! the previous install folder entirely.

use std::fs::{self, File};
use std::io::{self, BufReader, Read, Seek};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::Error;

/// itch.io-style manifest for an installed LewdZone app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppJson {
    pub format: String,
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_id: Option<i64>,
    pub title: String,
    pub version: String,
    pub platform: String,
    pub tab: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,
    pub download_url: String,
    pub install_path: String,
    pub installed_at: String,
    pub candidates: Vec<String>,
    pub launch_exe: String,
}

/// Input needed to build an `app.json` manifest.
#[derive(Debug, Clone, Copy)]
pub struct InstallMeta<'a> {
    pub slug: &'a str,
    pub post_id: Option<i64>,
    pub title: &'a str,
    pub version: &'a str,
    pub platform: &'a str,
    pub tab: &'a str,
    pub engine: Option<&'a str>,
    pub download_url: &'a str,
}

/// Extract `archive` into `install_dir`, remove the archive on success, and
/// write an `app.json` manifest. If `install_dir` already exists it is removed
/// first so updates replace the previous version rather than cluttering the
/// lzapps folder with multiple copies.
pub fn install_from_archive(
    archive: &Path,
    install_dir: &Path,
    meta: &InstallMeta<'_>,
) -> Result<AppJson, Error> {
    if !archive.exists() {
        return Err(Error::Usage(format!(
            "archive missing: {}",
            archive.display()
        )));
    }

    if install_dir.exists() {
        fs::remove_dir_all(install_dir).map_err(|e| {
            Error::Runtime(format!(
                "cannot replace old install dir {}: {e}",
                install_dir.display()
            ))
        })?;
    }
    fs::create_dir_all(install_dir).map_err(|e| {
        Error::Runtime(format!(
            "cannot create install dir {}: {e}",
            install_dir.display()
        ))
    })?;

    extract_zip(archive, install_dir)?;

    fs::remove_file(archive).map_err(|e| {
        Error::Runtime(format!(
            "extracted {} but could not remove archive: {e}",
            archive.display()
        ))
    })?;

    let candidates = find_candidates(install_dir);
    let manifest = AppJson {
        format: "lewdzone-lzapp".to_string(),
        slug: meta.slug.to_string(),
        post_id: meta.post_id,
        title: meta.title.to_string(),
        version: meta.version.to_string(),
        platform: meta.platform.to_string(),
        tab: meta.tab.to_string(),
        engine: meta.engine.map(String::from),
        download_url: meta.download_url.to_string(),
        install_path: install_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(meta.slug)
            .to_string(),
        installed_at: chrono::Utc::now().to_rfc3339(),
        candidates: candidates.clone(),
        launch_exe: String::new(),
    };

    let manifest_path = install_dir.join("app.json");
    let file = File::create(&manifest_path)
        .map_err(|e| Error::Runtime(format!("cannot create {}: {e}", manifest_path.display())))?;
    serde_json::to_writer_pretty(file, &manifest)
        .map_err(|e| Error::Runtime(format!("cannot write {}: {e}", manifest_path.display())))?;

    Ok(manifest)
}

fn extract_zip(archive: &Path, install_dir: &Path) -> Result<(), Error> {
    let mut file = File::open(archive)
        .map_err(|e| Error::Runtime(format!("cannot open archive {}: {e}", archive.display())))?;

    let mut magic = [0u8; 4];
    let bytes_read = file.read(&mut magic).unwrap_or(0);
    if bytes_read >= 4
        && &magic != b"PK\x03\x04"
        && &magic != b"PK\x05\x06"
        && &magic != b"PK\x07\x08"
    {
        let mut sample = vec![0u8; 512];
        sample[..4].copy_from_slice(&magic);
        let extra = file.read(&mut sample[4..]).unwrap_or(0);
        let sample_str = String::from_utf8_lossy(&sample[..4 + extra]).to_lowercase();
        if sample_str.contains("<!doctype")
            || sample_str.contains("<html")
            || sample_str.contains("<head")
        {
            return Err(Error::Runtime(
                "downloaded archive is an HTML webpage rather than a valid zip file (host requires interactive browser download)".to_string(),
            ));
        }
    }
    file.seek(std::io::SeekFrom::Start(0))
        .map_err(|e| Error::Runtime(format!("cannot seek archive {}: {e}", archive.display())))?;

    let reader = BufReader::new(file);
    let mut zip = zip::ZipArchive::new(reader)
        .map_err(|e| Error::Runtime(format!("cannot read zip {}: {e}", archive.display())))?;

    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|e| {
            Error::Runtime(format!(
                "zip entry {i} of {} failed: {e}",
                archive.display()
            ))
        })?;
        let rel = match entry.enclosed_name() {
            Some(p) => p.to_path_buf(),
            None => continue,
        };
        let outpath = install_dir.join(&rel);

        if entry.is_dir() {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out = File::create(&outpath)
                .map_err(|e| Error::Runtime(format!("cannot create {}: {e}", outpath.display())))?;
            io::copy(&mut entry, &mut out).map_err(|e| {
                Error::Runtime(format!("cannot extract {}: {e}", outpath.display()))
            })?;
        }
    }

    Ok(())
}

fn find_candidates(install_dir: &Path) -> Vec<String> {
    let mut candidates = Vec::new();
    let mut stack = vec![install_dir.to_path_buf()];

    while let Some(cur) = stack.pop() {
        let entries = match fs::read_dir(&cur) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let rel = path
                .strip_prefix(install_dir)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_default();
            if rel.is_empty() {
                continue;
            }

            if path.is_dir() {
                #[cfg(target_os = "macos")]
                if path.extension().and_then(|e| e.to_str()) == Some("app") {
                    candidates.push(rel);
                    continue;
                }
                stack.push(path);
                continue;
            }

            if is_executable_candidate(&path) {
                candidates.push(rel);
            }
        }
    }

    candidates.sort();
    candidates
}

fn is_executable_candidate(path: &Path) -> bool {
    #[cfg(target_os = "windows")]
    {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| matches!(e.to_ascii_lowercase().as_str(), "exe" | "bat" | "cmd"))
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(path)
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;

    fn temp_dir(tag: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!(
            "lewdzone-extract-test-{}-{}",
            tag,
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    fn write_zip(dir: &Path, name: &str, files: &[(&str, &[u8])]) -> PathBuf {
        let path = dir.join(name);
        let file = File::create(&path).unwrap();
        let mut zip = zip::write::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        for (name, data) in files {
            zip.start_file(*name, options).unwrap();
            zip.write_all(data).unwrap();
        }
        zip.finish().unwrap();
        path
    }

    #[test]
    fn extracts_zip_and_writes_manifest() {
        let tmp = temp_dir("extract");
        let archive = write_zip(
            &tmp,
            "game.zip",
            &[
                ("game.exe", b"fake exe"),
                ("readme.txt", b"hello"),
                ("data/level1.bin", b"level"),
            ],
        );
        let install = tmp.join("treasure-of-nadia");

        let meta = InstallMeta {
            slug: "treasure-of-nadia",
            post_id: Some(123),
            title: "Treasure of Nadia",
            version: "1.0",
            platform: "pc",
            tab: "fileknot",
            engine: Some("Ren'Py"),
            download_url: "https://fileknot.io/dl/abc",
        };

        let manifest = install_from_archive(&archive, &install, &meta).unwrap();

        assert!(install.join("game.exe").exists());
        assert!(install.join("readme.txt").exists());
        assert!(install.join("data/level1.bin").exists());
        assert!(!archive.exists(), "archive removed after extraction");
        assert_eq!(manifest.slug, "treasure-of-nadia");
        assert_eq!(manifest.post_id, Some(123));
        assert_eq!(manifest.version, "1.0");
        assert_eq!(manifest.platform, "pc");
        assert_eq!(manifest.engine.as_deref(), Some("Ren'Py"));
        assert!(install.join("app.json").exists());

        #[cfg(target_os = "windows")]
        assert!(manifest.candidates.contains(&"game.exe".to_string()));
        assert!(manifest.launch_exe.is_empty());
    }

    #[test]
    fn replaces_existing_install_dir() {
        let tmp = temp_dir("replace");
        let archive = write_zip(&tmp, "game.zip", &[("new.exe", b"new")]);
        let install = tmp.join("treasure-of-nadia");
        fs::create_dir_all(&install).unwrap();
        fs::write(install.join("old.exe"), b"old").unwrap();

        let meta = InstallMeta {
            slug: "treasure-of-nadia",
            post_id: None,
            title: "Treasure of Nadia",
            version: "2.0",
            platform: "pc",
            tab: "fileknot",
            engine: None,
            download_url: "https://fileknot.io/dl/xyz",
        };

        install_from_archive(&archive, &install, &meta).unwrap();
        assert!(!install.join("old.exe").exists());
        assert!(install.join("new.exe").exists());
    }

    #[test]
    fn errors_when_archive_missing() {
        let tmp = temp_dir("missing");
        let missing = tmp.join("nope.zip");
        let meta = InstallMeta {
            slug: "x",
            post_id: None,
            title: "X",
            version: "1",
            platform: "pc",
            tab: "fileknot",
            engine: None,
            download_url: "u",
        };
        assert!(install_from_archive(&missing, &tmp.join("x"), &meta).is_err());
    }
}
