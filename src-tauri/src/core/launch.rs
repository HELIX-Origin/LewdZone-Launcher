//! `launch` — start an installed game from its `lzapps/<slug>/app.json` manifest.

use std::path::PathBuf;
use std::process::Command;

use crate::core::{Context, Error};

pub fn locate_manifest(
    ctx: &Context,
    game: &str,
) -> Result<(crate::core::extract::AppJson, PathBuf), Error> {
    // 1. Try finding in list_installed (finds installed/<engine>/<slug>/app.json)
    if let Ok(installed) = crate::core::library::list_installed(ctx) {
        let sanitized = crate::core::folder::sanitize_segment(game);
        if let Some(app) = installed.into_iter().find(|a| {
            a.slug.eq_ignore_ascii_case(game)
                || a.slug.eq_ignore_ascii_case(&sanitized)
                || a.title.eq_ignore_ascii_case(game)
        }) {
            let manifest_path = app.install_path.join("app.json");
            if manifest_path.is_file() {
                if let Ok(raw) = std::fs::read_to_string(&manifest_path) {
                    if let Ok(manifest) =
                        serde_json::from_str::<crate::core::extract::AppJson>(&raw)
                    {
                        return Ok((manifest, app.install_path));
                    }
                }
            }
        }
    }

    // 2. Direct lzapps/installed root lookup fallback
    let lzapps = crate::core::folder::lzapps_root(ctx)?;
    let install_dir = lzapps.join(crate::core::folder::sanitize_segment(game));
    let manifest_path = install_dir.join("app.json");
    if !manifest_path.exists() {
        return Err(Error::Usage(format!(
            "'{game}' is not installed in {}",
            lzapps.display()
        )));
    }

    let raw = std::fs::read_to_string(&manifest_path)?;
    let manifest: crate::core::extract::AppJson = serde_json::from_str(&raw)
        .map_err(|e| Error::Runtime(format!("corrupt app.json for {game}: {e}")))?;

    let custom_path = PathBuf::from(&manifest.install_path);
    let base_dir = if custom_path.is_absolute() && custom_path.exists() {
        custom_path
    } else {
        install_dir
    };

    Ok((manifest, base_dir))
}

pub fn run(ctx: &Context, game: &str) -> Result<crate::cli::ExitCode, Error> {
    let (manifest, base_dir) = locate_manifest(ctx, game)?;

    let exe_rel = if manifest.launch_exe.trim().is_empty() {
        manifest.candidates.first().map(|s| s.as_str())
    } else {
        Some(manifest.launch_exe.trim())
    }
    .ok_or_else(|| Error::Runtime(format!("no executable candidate found for {game}")))?;

    let exe_path = base_dir.join(exe_rel);
    if !exe_path.exists() {
        return Err(Error::Runtime(format!(
            "launch target not found: {}",
            exe_path.display()
        )));
    }

    // 1. Record launch in SQLite database (increments play_count, updates last_played_at)
    let slug = manifest.slug.clone();
    if let Ok(conn) = crate::db::open(&ctx.db_path) {
        let _ = crate::db::migrate(&conn);
        let _ = crate::db::repo::record_game_launch(&conn, &slug);
    }

    // 2. Spawn the game process and monitor session playtime in a detached thread
    let db_path = ctx.db_path.clone();
    let is_mac_app =
        cfg!(target_os = "macos") && exe_path.extension().and_then(|e| e.to_str()) == Some("app");

    let mut cmd = if is_mac_app {
        let mut c = Command::new("open");
        c.arg(&exe_path);
        c
    } else {
        Command::new(&exe_path)
    };

    cmd.current_dir(&base_dir);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| Error::Runtime(format!("failed to launch {}: {e}", exe_path.display())))?;

    // Background thread monitors child exit to record elapsed playtime
    std::thread::spawn(move || {
        let start = std::time::Instant::now();
        let _ = child.wait();
        let elapsed_secs = start.elapsed().as_secs() as i64;
        if elapsed_secs > 0 {
            if let Ok(conn) = crate::db::open(&db_path) {
                let _ = crate::db::repo::add_game_playtime(&conn, &slug, elapsed_secs);
            }
        }
    });

    Ok(crate::cli::ExitCode::Ok)
}

/// Resolve the executable that would be launched for `slug` without running it.
pub fn resolve_exe(ctx: &Context, game: &str) -> Result<PathBuf, Error> {
    let lzapps = crate::core::folder::lzapps_root(ctx)?;
    let install_dir = lzapps.join(crate::core::folder::sanitize_segment(game));
    let manifest_path = install_dir.join("app.json");
    if !manifest_path.exists() {
        return Err(Error::Usage(format!(
            "'{game}' is not installed in {}",
            lzapps.display()
        )));
    }

    let raw = std::fs::read_to_string(&manifest_path)?;
    let manifest: crate::core::extract::AppJson = serde_json::from_str(&raw)
        .map_err(|e| Error::Runtime(format!("corrupt app.json for {game}: {e}")))?;

    let custom_path = PathBuf::from(&manifest.install_path);
    let base_dir = if custom_path.is_absolute() && custom_path.exists() {
        custom_path
    } else {
        install_dir
    };

    let exe_rel = if manifest.launch_exe.trim().is_empty() {
        manifest.candidates.first().map(|s| s.as_str())
    } else {
        Some(manifest.launch_exe.trim())
    }
    .ok_or_else(|| Error::Runtime(format!("no executable candidate found for {game}")))?;

    Ok(base_dir.join(exe_rel))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::extract::AppJson;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn temp_ctx(tag: &str) -> (Context, PathBuf) {
        let tmp = std::env::temp_dir().join(format!(
            "lewdzone-launch-test-{}-{}",
            tag,
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let db = tmp.join(format!("{tag}.db"));
        let cfg = tmp.join(format!("{tag}.json"));
        let lib_root = tmp.to_string_lossy();
        let cfg_json = serde_json::json!({"library_root": lib_root.to_string()});
        fs::write(&cfg, serde_json::to_string(&cfg_json).unwrap()).unwrap();
        (Context::new(db, cfg), tmp)
    }

    fn write_manifest(tmp: &Path, slug: &str, launch_exe: &str, candidates: &[&str]) -> PathBuf {
        let install = tmp.join("lzapps").join(slug);
        fs::create_dir_all(&install).unwrap();
        let manifest = AppJson {
            format: "lewdzone-lzapp".to_string(),
            slug: slug.to_string(),
            post_id: Some(42),
            title: "Test Game".to_string(),
            version: "1.0".to_string(),
            platform: "pc".to_string(),
            tab: "fileknot".to_string(),
            engine: Some("Unity".to_string()),
            download_url: "https://fileknot.io/dl/x".to_string(),
            install_path: slug.to_string(),
            installed_at: chrono::Utc::now().to_rfc3339(),
            candidates: candidates.iter().map(|s| s.to_string()).collect(),
            launch_exe: launch_exe.to_string(),
        };
        let path = install.join("app.json");
        fs::write(&path, serde_json::to_string_pretty(&manifest).unwrap()).unwrap();
        install
    }

    #[test]
    fn resolve_exe_uses_launch_exe_override() {
        let (ctx, tmp) = temp_ctx("override");
        let install = write_manifest(&tmp, "test-game", "custom/launcher.exe", &["game.exe"]);
        fs::create_dir_all(install.join("custom")).unwrap();
        fs::write(install.join("custom").join("launcher.exe"), b"x").unwrap();

        let exe = resolve_exe(&ctx, "test-game").unwrap();
        assert_eq!(
            exe.file_name().and_then(|n| n.to_str()),
            Some("launcher.exe")
        );
    }

    #[test]
    fn resolve_exe_falls_back_to_first_candidate() {
        let (ctx, tmp) = temp_ctx("candidate");
        let install = write_manifest(&tmp, "test-game", "", &["game.exe"]);
        fs::write(install.join("game.exe"), b"x").unwrap();

        let exe = resolve_exe(&ctx, "test-game").unwrap();
        assert_eq!(exe.file_name().and_then(|n| n.to_str()), Some("game.exe"));
    }

    #[test]
    fn resolve_exe_errors_when_not_installed() {
        let (ctx, tmp) = temp_ctx("missing");
        // Ensure lzapps dir exists but empty
        fs::create_dir_all(tmp.join("lzapps")).unwrap();
        assert!(resolve_exe(&ctx, "missing-game").is_err());
    }

    #[test]
    fn resolve_exe_errors_when_no_candidates() {
        let (ctx, tmp) = temp_ctx("nocand");
        write_manifest(&tmp, "test-game", "", &[]);
        assert!(resolve_exe(&ctx, "test-game").is_err());
    }
}
