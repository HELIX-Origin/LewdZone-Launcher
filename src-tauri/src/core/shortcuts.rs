//! Native per-OS shortcuts for installed games (owner: shortcuts family).
//!
//! Creates a Desktop shortcut pointing at the resolved executable for a game in
//! `lzapps/<slug>/`. SteamGridDB artwork for the icon is optional and left as a
//! future enhancement (Phase 4 polish).

use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::ExitCode;
use crate::core::{folder, launch, Context, Error};

/// Create a Desktop shortcut for an installed game and return the path to it.
pub fn create(ctx: &Context, slug: &str) -> Result<PathBuf, Error> {
    let exe = launch::resolve_exe(ctx, slug)?;

    let install = folder::install_dir(&folder::lzapps_root(ctx)?, slug);
    let title = shortcut_title(&install, slug);
    let desktop = dirs::desktop_dir()
        .ok_or_else(|| Error::Runtime("cannot resolve the Desktop directory".to_string()))?;

    create_for_platform(&desktop, &title, slug, &exe, &install)
}

/// Build a human title for the shortcut, sanitizing forbidden characters.
fn shortcut_title(install_dir: &Path, fallback: &str) -> String {
    if let Some(manifest) = read_manifest(install_dir) {
        folder::sanitize_segment(&manifest.title)
    } else {
        folder::sanitize_segment(fallback)
    }
}

fn read_manifest(install_dir: &Path) -> Option<crate::core::extract::AppJson> {
    let path = install_dir.join("app.json");
    let bytes = fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

#[cfg(target_os = "windows")]
fn create_for_platform(
    desktop: &Path,
    title: &str,
    _slug: &str,
    exe: &Path,
    _install_dir: &Path,
) -> Result<PathBuf, Error> {
    let out = desktop.join(format!("{title}.lnk"));
    let target = exe.to_string_lossy().replace('\'', "''");
    let cwd = exe
        .parent()
        .unwrap_or(Path::new("."))
        .to_string_lossy()
        .replace('\'', "''");
    let out_escaped = out.to_string_lossy().replace('\'', "''");

    let ps = format!(
        "$s=(New-Object -ComObject WScript.Shell).CreateShortcut('{out_escaped}');\
         $s.TargetPath='{target}';\
         $s.WorkingDirectory='{cwd}';\
         $s.IconLocation='{target},0';\
         $s.Save()"
    );

    let status = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps])
        .status()?;

    if !status.success() {
        return Err(Error::Runtime(format!(
            "failed to create Windows shortcut for {title}"
        )));
    }
    Ok(out)
}

#[cfg(target_os = "linux")]
fn create_for_platform(
    desktop: &Path,
    title: &str,
    slug: &str,
    exe: &Path,
    install_dir: &Path,
) -> Result<PathBuf, Error> {
    let out = desktop.join(format!("{title}.desktop"));
    let exec = exe.to_string_lossy();
    let icon = icon_path(install_dir, slug)
        .unwrap_or_else(|| exe.to_path_buf())
        .to_string_lossy()
        .to_string();

    let contents = format!(
        "[Desktop Entry]\n\
         Name={title}\n\
         Comment=Play {title} with LewdZone Launcher\n\
         Exec={exec}\n\
         Icon={icon}\n\
         Type=Application\n\
         Terminal=false\n\
         Categories=Game;\n"
    );

    fs::write(&out, contents)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&out)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&out, perms)?;
    }
    Ok(out)
}

#[cfg(target_os = "macos")]
fn create_for_platform(
    desktop: &Path,
    title: &str,
    slug: &str,
    exe: &Path,
    install_dir: &Path,
) -> Result<PathBuf, Error> {
    let app_name = format!("{title}.app");
    let app_bundle = desktop.join(&app_name);
    let contents = app_bundle.join("Contents");
    let macos = contents.join("MacOS");
    fs::create_dir_all(&macos)?;

    let launcher = macos.join(slug);
    let exec = exe.to_string_lossy();
    let script = format!("#!/bin/sh\nexec \"{exec}\" \"$@\"\n");
    fs::write(&launcher, script)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&launcher)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&launcher, perms)?;
    }

    let icon = icon_path(install_dir, slug)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let plist = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\">\n\
         <dict>\n\
         <key>CFBundleName</key><string>{title}</string>\n\
         <key>CFBundleExecutable</key><string>{slug}</string>\n\
         <key>CFBundleIdentifier</key><string>com.lewdzone.launcher.{slug}</string>\n\
         <key>CFBundlePackageType</key><string>APPL</string>\n\
         <key>CFBundleIconFile</key><string>{icon}</string>\n\
         </dict>\n\
         </plist>\n"
    );
    fs::write(contents.join("Info.plist"), plist)?;

    Ok(app_bundle)
}

/// Best-effort icon path inside the install directory.
#[allow(dead_code)]
fn icon_path(install_dir: &Path, _slug: &str) -> Option<PathBuf> {
    // Future: use cached SteamGridDB icon here.
    // For now, prefer an .ico or .png sitting next to the executable.
    for ext in ["ico", "png"] {
        let candidate = install_dir.join(format!("icon.{ext}"));
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

/// CLI entry point for `lewdzone shortcuts [--game <slug>]`.
pub fn run(ctx: &Context, game: Option<&str>, _skip_artwork: bool) -> Result<ExitCode, Error> {
    let slug = game.ok_or_else(|| {
        Error::Usage("--game <slug> is required to create a shortcut".to_string())
    })?;
    let path = create(ctx, slug)?;
    println!("{}", path.display());
    Ok(ExitCode::Ok)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn tmp_ctx(tag: &str) -> (Context, PathBuf, PathBuf) {
        let pid = std::process::id();
        let dir = std::env::temp_dir().join(format!("lz-sc-{tag}-{pid}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let db = dir.join("lewdzone.db");
        let config = dir.join("config.json");
        let ctx = Context::new(db, config.clone());

        // Point library-root at our temp dir so lzapps_root resolves under it.
        let settings = crate::core::settings::Settings::default();
        settings.save(&config).expect("temp settings should save");
        let mut raw: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&config).unwrap()).unwrap();
        raw["library_root"] = serde_json::Value::String(dir.to_string_lossy().to_string());
        fs::write(&config, serde_json::to_string_pretty(&raw).unwrap()).unwrap();

        (ctx, dir.clone(), dir.join("lzapps"))
    }

    fn fake_install(lzapps: &Path, slug: &str, candidate: &str) {
        let install = folder::install_dir(lzapps, slug);
        fs::create_dir_all(&install).unwrap();
        let exe = install.join(candidate);
        let mut f = fs::File::create(&exe).unwrap();
        f.write_all(b"#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&exe).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&exe, perms).unwrap();
        }
        let manifest = crate::core::extract::AppJson {
            format: "lewdzone-lzapp".to_string(),
            slug: slug.to_string(),
            post_id: None,
            title: slug.to_string(),
            version: "v1".to_string(),
            platform: "pc".to_string(),
            tab: "official".to_string(),
            engine: None,
            download_url: "https://example.com/dl".to_string(),
            install_path: install.to_string_lossy().to_string(),
            installed_at: "2026-01-01T00:00:00Z".to_string(),
            candidates: vec![candidate.to_string()],
            launch_exe: "".to_string(),
        };
        fs::write(
            install.join("app.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
    }

    #[test]
    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
    fn creates_desktop_shortcut() {
        let (ctx, _root, lzapps) = tmp_ctx("create");
        fake_install(&lzapps, "wild-life", "WildLife.exe");

        let out = create(&ctx, "wild-life").unwrap();
        assert!(out.exists(), "shortcut should exist at {}", out.display());
    }
}
