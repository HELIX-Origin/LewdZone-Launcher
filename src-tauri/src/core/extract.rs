//! Extraction and installed-app manifest writer.
//!
//! After a direct-file download finishes, `.zip` archives are extracted into
//! `<lzapps_root>/<slug>/` and an itch.io-style `app.json` manifest is written
//! so the launcher can launch and manage the installed app. Updates replace
//! the previous install folder entirely.

use std::fs::{self, File};
use std::io::{self, BufReader, Read, Seek};
use std::path::{Path, PathBuf};

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
    install_from_archive_with_progress(archive, install_dir, meta, &mut |_, _| Ok(()))
}

pub fn install_from_archive_with_progress(
    archive: &Path,
    install_dir: &Path,
    meta: &InstallMeta<'_>,
    progress: &mut dyn FnMut(u64, u64) -> Result<(), Error>,
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

    extract_archive_with_progress(archive, install_dir, progress)?;

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
        candidates,
        launch_exe: String::new(),
    };

    let manifest_path = install_dir.join("app.json");
    let file = File::create(&manifest_path)
        .map_err(|e| Error::Runtime(format!("cannot create {}: {e}", manifest_path.display())))?;
    serde_json::to_writer_pretty(file, &manifest)
        .map_err(|e| Error::Runtime(format!("cannot write {}: {e}", manifest_path.display())))?;

    Ok(manifest)
}

pub fn extract_zip(archive: &Path, install_dir: &Path) -> Result<(), Error> {
    extract_archive_with_progress(archive, install_dir, &mut |_, _| Ok(()))
}

/// Helper to probe a candidate path (which may be a file or a directory containing 7za.exe / 7z.exe)
fn probe_7z_path(path: &Path) -> Option<PathBuf> {
    if path.is_file() {
        return Some(path.to_path_buf());
    }
    if path.is_dir() {
        for sub in &["x64/7za.exe", "7za.exe", "7z.exe", "x64/7za", "7za", "7z"] {
            let candidate = path.join(sub);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Resolve the path to the 7-Zip console executable (`7za.exe`, `7z.exe`, or `7za`).
pub fn resolve_7z_path(custom: Option<&Path>) -> Option<PathBuf> {
    if let Some(p) = custom {
        if let Some(found) = probe_7z_path(p) {
            return Some(found);
        }
    }

    if let Some(cfg_path) = crate::core::paths::default_config_path() {
        if let Ok(settings) = crate::core::settings::Settings::load(&cfg_path) {
            if let Some(val) = settings.get_value("7z-path") {
                if let Some(s) = val.as_str() {
                    let p = Path::new(s.trim());
                    if let Some(found) = probe_7z_path(p) {
                        return Some(found);
                    }
                }
            }
        }
    }

    // Check C:\Utilities\7z (user's portable tools location)
    let user_tools = Path::new(r"C:\Utilities\7z");
    if let Some(found) = probe_7z_path(user_tools) {
        return Some(found);
    }

    // Check standard Windows program file paths
    for cand in &[
        r"C:\Program Files\7-Zip\7z.exe",
        r"C:\Program Files (x86)\7-Zip\7z.exe",
    ] {
        let p = Path::new(cand);
        if p.is_file() {
            return Some(p.to_path_buf());
        }
    }

    // Test running "7za" or "7z" directly from PATH:
    for bin in &["7za", "7z"] {
        let mut cmd = std::process::Command::new(bin);
        cmd.arg("-h");
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x0800_0000);
        }
        if let Ok(status) = cmd.status() {
            if status.success() {
                return Some(PathBuf::from(bin));
            }
        }
    }

    None
}

/// Extract an archive using the 7-Zip console executable (`7za` or `7z`).
/// Streams stdout with `-bsp1` to compute real-time byte progress.
pub fn extract_with_7z(
    seven_zip_bin: &Path,
    archive: &Path,
    install_dir: &Path,
    progress: &mut dyn FnMut(u64, u64) -> Result<(), Error>,
) -> Result<(), Error> {
    let file_size = fs::metadata(archive).map(|m| m.len()).unwrap_or(0);
    progress(0, file_size)?;

    let mut cmd = std::process::Command::new(seven_zip_bin);
    cmd.arg("x")
        .arg("-y")
        .arg("-bsp1")
        .arg(format!("-o{}", install_dir.display()))
        .arg(archive)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }

    let mut child = cmd.spawn().map_err(|e| {
        Error::Runtime(format!(
            "failed to spawn 7z ({}): {e}",
            seven_zip_bin.display()
        ))
    })?;

    let stdout = child.stdout.take();
    if let Some(stdout) = stdout {
        use std::io::Read;
        let mut reader = std::io::BufReader::new(stdout);
        let mut buf = [0u8; 256];
        let mut num_buf = String::new();
        let mut last_pct = 0u64;

        while let Ok(n) = reader.read(&mut buf) {
            if n == 0 {
                break;
            }
            for &byte in &buf[..n] {
                if byte.is_ascii_digit() {
                    num_buf.push(byte as char);
                } else if byte == b'%' {
                    if let Ok(pct) = num_buf.parse::<u64>() {
                        if pct <= 100 && pct != last_pct {
                            last_pct = pct;
                            let done = (pct * file_size) / 100;
                            if let Err(e) = progress(done, file_size) {
                                let _ = child.kill();
                                return Err(e);
                            }
                        }
                    }
                    num_buf.clear();
                } else if byte == b'\r' || byte == b'\n' || byte == b' ' {
                    num_buf.clear();
                }
            }
        }
    }

    let status = child
        .wait()
        .map_err(|e| Error::Runtime(format!("failed waiting for 7z extraction: {e}")))?;

    if status.success() {
        progress(file_size, file_size)?;
        Ok(())
    } else {
        Err(Error::Runtime(format!(
            "7z extraction failed with exit code {:?}",
            status.code()
        )))
    }
}

/// Extract any supported archive format (zip, 7z, rar, tar, tar.gz, sfx .exe) into `install_dir`.
pub fn extract_archive_with_progress(
    archive: &Path,
    install_dir: &Path,
    progress: &mut dyn FnMut(u64, u64) -> Result<(), Error>,
) -> Result<(), Error> {
    // 1. Try 7-Zip console executable first (faster, supports all formats, multi-threaded):
    if let Some(seven_zip) = resolve_7z_path(None) {
        match extract_with_7z(&seven_zip, archive, install_dir, progress) {
            Ok(()) => return Ok(()),
            Err(e) => {
                if matches!(e, Error::Usage(ref msg) if msg.contains("cancelled")) {
                    return Err(e);
                }
                // Fall back to native/system extractors
            }
        }
    }

    let name = archive
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    // 2. If it's a .zip file, try our native zip extractor first.
    if name.ends_with(".zip") {
        match extract_zip_with_progress(archive, install_dir, progress) {
            Ok(()) => return Ok(()),
            Err(e) => {
                // If it's a cancellation error, do NOT swallow it or fall back to system tool!
                if matches!(e, Error::Usage(ref msg) if msg.contains("cancelled")) {
                    return Err(e);
                }
                // If native zip fails (e.g. invalid zip / missing EOCD or 7z-compressed zip),
                // fall back to system tool.
            }
        }
    }

    // 3. For tar, tar.gz, 7z, rar, sfx, and zip fallbacks:
    extract_via_system_tool(archive, install_dir, progress)
}

/// System-level extractor using `tar` (bsdtar, built into Windows/macOS/Linux),
/// `7z`/`7za`, or native SFX execution.
pub fn extract_via_system_tool(
    archive: &Path,
    install_dir: &Path,
    progress: &mut dyn FnMut(u64, u64) -> Result<(), Error>,
) -> Result<(), Error> {
    let file_size = fs::metadata(archive).map(|m| m.len()).unwrap_or(0);
    progress(0, file_size)?;

    // 1. Try `tar` (bsdtar): natively supports zip, tar, tar.gz, tar.bz2, tar.xz, 7z, rar, and sfx
    let mut tar_cmd = std::process::Command::new("tar");
    tar_cmd.arg("-xf").arg(archive).arg("-C").arg(install_dir);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        tar_cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }

    if let Ok(output) = tar_cmd.output() {
        if output.status.success() {
            progress(file_size, file_size)?;
            return Ok(());
        }
    }

    // 2. Try `7z` or `7za` if available
    for bin in &["7z", "7za"] {
        let mut cmd = std::process::Command::new(bin);
        cmd.arg("x")
            .arg("-y")
            .arg(format!("-o{}", install_dir.display()))
            .arg(archive);

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x0800_0000);
        }

        if let Ok(output) = cmd.output() {
            if output.status.success() {
                progress(file_size, file_size)?;
                return Ok(());
            }
        }
    }

    // 3. If it's a self-extracting .exe archive on Windows:
    #[cfg(target_os = "windows")]
    {
        let name = archive
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        if name.ends_with(".exe") {
            use std::os::windows::process::CommandExt;
            let mut sfx_cmd = std::process::Command::new(archive);
            sfx_cmd
                .arg("-y")
                .arg(format!("-o{}", install_dir.display()))
                .current_dir(install_dir)
                .creation_flags(0x0800_0000);
            if let Ok(output) = sfx_cmd.output() {
                if output.status.success() {
                    progress(file_size, file_size)?;
                    return Ok(());
                }
            }
        }
    }

    Err(Error::Runtime(format!(
        "could not extract archive {}: unsupported format or extraction command failed",
        archive.display()
    )))
}

pub fn extract_zip_with_progress(
    archive: &Path,
    install_dir: &Path,
    progress: &mut dyn FnMut(u64, u64) -> Result<(), Error>,
) -> Result<(), Error> {
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

    let total_bytes: u64 = (0..zip.len())
        .filter_map(|i| zip.by_index(i).ok().map(|e| e.size()))
        .sum();
    let mut bytes_done: u64 = 0;

    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|e| {
            Error::Runtime(format!(
                "zip entry {i} of {} failed: {e}",
                archive.display()
            ))
        })?;
        let entry_size = entry.size();
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
        bytes_done += entry_size;
        progress(bytes_done, total_bytes)?;
    }

    Ok(())
}

/// Extract an archive into `target_dir` with progress, write manifest to `lzapps/<slug>/app.json`
/// pointing to `target_dir`, and do NOT delete the source archive.
pub fn install_from_custom_archive(
    archive: &Path,
    target_dir: &Path,
    meta: &InstallMeta<'_>,
    lzapps_dir: &Path,
    progress: &mut dyn FnMut(u64, u64) -> Result<(), Error>,
) -> Result<AppJson, Error> {
    if !archive.exists() {
        return Err(Error::Usage(format!(
            "archive missing: {}",
            archive.display()
        )));
    }

    fs::create_dir_all(target_dir).map_err(|e| {
        Error::Runtime(format!(
            "cannot create target dir {}: {e}",
            target_dir.display()
        ))
    })?;

    extract_archive_with_progress(archive, target_dir, progress)?;

    let candidates = find_candidates(target_dir);
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
        install_path: target_dir.display().to_string(),
        installed_at: chrono::Utc::now().to_rfc3339(),
        candidates: candidates.clone(),
        launch_exe: String::new(),
    };

    let manifest_dir = lzapps_dir.join(crate::core::folder::sanitize_segment(meta.slug));
    fs::create_dir_all(&manifest_dir).map_err(|e| {
        Error::Runtime(format!(
            "cannot create manifest dir {}: {e}",
            manifest_dir.display()
        ))
    })?;
    let manifest_path = manifest_dir.join("app.json");
    let file = File::create(&manifest_path)
        .map_err(|e| Error::Runtime(format!("cannot create {}: {e}", manifest_path.display())))?;
    serde_json::to_writer_pretty(file, &manifest)
        .map_err(|e| Error::Runtime(format!("cannot write {}: {e}", manifest_path.display())))?;

    let local_manifest = target_dir.join("app.json");
    if let Ok(f) = File::create(&local_manifest) {
        let _ = serde_json::to_writer_pretty(f, &manifest);
    }

    Ok(manifest)
}

pub fn find_candidates(install_dir: &Path) -> Vec<String> {
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

    #[test]
    fn probe_and_resolve_7z_path() {
        let fake_dir = temp_dir("7z_probe");
        let fake_bin = fake_dir.join("7za.exe");
        fs::write(&fake_bin, b"fake 7za binary").unwrap();

        // Testing direct file path
        let resolved = resolve_7z_path(Some(&fake_bin));
        assert_eq!(resolved, Some(fake_bin.clone()));

        // Testing folder containing 7za.exe
        let resolved_dir = resolve_7z_path(Some(&fake_dir));
        assert_eq!(resolved_dir, Some(fake_bin));
    }
}
