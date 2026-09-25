//! Native per-OS shortcuts for installed games.
//!
//! Generates working desktop shortcuts and start menu entries:
//! - Windows: `.lnk` shortcut files using PowerShell / Windows Script Host COM automation,
//!   pointing to the game executable or launcher, using the executable's embedded icon.
//! - Linux: `.desktop` FreeDesktop entries in `~/.local/share/applications/` and `~/Desktop/`.
//! - macOS: `.command` / application alias in `~/Applications/` or `~/Desktop/`.

use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::ExitCode;
use crate::core::{Context, Error};

/// Target options for shortcut creation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShortcutOptions {
    pub desktop: bool,
    pub start_menu: bool,
}

impl Default for ShortcutOptions {
    fn default() -> Self {
        Self {
            desktop: true,
            start_menu: false,
        }
    }
}

/// Sanitize filename characters for safe filesystem paths.
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

/// Create native shortcuts for an installed game.
pub fn create(ctx: &Context, slug: &str, opts: ShortcutOptions) -> Result<Vec<PathBuf>, Error> {
    let (manifest, base_dir) = crate::core::launch::locate_manifest(ctx, slug)?;

    let exe_rel = if manifest.launch_exe.trim().is_empty() {
        manifest.candidates.first().map(|s| s.as_str())
    } else {
        Some(manifest.launch_exe.trim())
    }
    .ok_or_else(|| Error::Runtime(format!("no executable candidate found for {slug}")))?;

    let exe_path = base_dir.join(exe_rel);
    if !exe_path.exists() {
        return Err(Error::Runtime(format!(
            "executable not found for {slug}: {}",
            exe_path.display()
        )));
    }

    let title = if manifest.title.trim().is_empty() {
        slug.to_string()
    } else {
        manifest.title.trim().to_string()
    };

    let mut created = Vec::new();

    #[cfg(target_os = "windows")]
    {
        if opts.desktop {
            if let Some(desktop_dir) = dirs::desktop_dir() {
                let lnk_path = desktop_dir.join(format!("{}.lnk", sanitize_filename(&title)));
                create_windows_shortcut(&lnk_path, &exe_path, &base_dir, &title)?;
                created.push(lnk_path);
            }
        }
        if opts.start_menu {
            if let Some(data_dir) = dirs::data_dir() {
                let programs_dir =
                    data_dir.join(r"Microsoft\Windows\Start Menu\Programs\LewdZone Games");
                let _ = fs::create_dir_all(&programs_dir);
                let lnk_path = programs_dir.join(format!("{}.lnk", sanitize_filename(&title)));
                create_windows_shortcut(&lnk_path, &exe_path, &base_dir, &title)?;
                created.push(lnk_path);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let content = format!(
            "[Desktop Entry]\nType=Application\nName={}\nExec=\"{}\"\nPath=\"{}\"\nTerminal=false\nCategories=Game;\n",
            title,
            exe_path.display(),
            base_dir.display()
        );

        if opts.desktop {
            if let Some(desktop_dir) = dirs::desktop_dir() {
                let desktop_file =
                    desktop_dir.join(format!("{}.desktop", sanitize_filename(&title)));
                fs::write(&desktop_file, &content)?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = fs::set_permissions(&desktop_file, fs::Permissions::from_mode(0o755));
                }
                created.push(desktop_file);
            }
        }
        if opts.start_menu {
            if let Some(data_dir) = dirs::data_dir() {
                let app_dir = data_dir.join("applications");
                let _ = fs::create_dir_all(&app_dir);
                let app_file =
                    app_dir.join(format!("lewdzone-{}.desktop", sanitize_filename(slug)));
                fs::write(&app_file, &content)?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = fs::set_permissions(&app_file, fs::Permissions::from_mode(0o755));
                }
                created.push(app_file);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if opts.desktop {
            if let Some(desktop_dir) = dirs::desktop_dir() {
                let cmd_file = desktop_dir.join(format!("{}.command", sanitize_filename(&title)));
                let script = format!(
                    "#!/bin/bash\ncd \"{}\" && \"{}\"\n",
                    base_dir.display(),
                    exe_path.display()
                );
                fs::write(&cmd_file, script)?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = fs::set_permissions(&cmd_file, fs::Permissions::from_mode(0o755));
                }
                created.push(cmd_file);
            }
        }
    }

    Ok(created)
}

#[cfg(target_os = "windows")]
fn create_windows_shortcut(lnk: &Path, exe: &Path, cwd: &Path, desc: &str) -> Result<(), Error> {
    use std::process::Command;
    let lnk_str = lnk.to_string_lossy().replace('\'', "''");
    let exe_str = exe.to_string_lossy().replace('\'', "''");
    let cwd_str = cwd.to_string_lossy().replace('\'', "''");
    let desc_str = desc.replace('\'', "''");

    let script = format!(
        "$ws = New-Object -ComObject WScript.Shell; \
         $s = $ws.CreateShortcut('{lnk_str}'); \
         $s.TargetPath = '{exe_str}'; \
         $s.WorkingDirectory = '{cwd_str}'; \
         $s.Description = '{desc_str}'; \
         $s.IconLocation = '{exe_str},0'; \
         $s.Save()"
    );

    let mut cmd = Command::new("powershell");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", &script]);
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    let output = cmd
        .output()
        .map_err(|e| Error::Runtime(format!("failed to run powershell for shortcut: {e}")))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(Error::Runtime(format!("shortcut creation failed: {err}")));
    }
    Ok(())
}

/// CLI entry point for `lewdzone shortcuts [--game <slug>]`.
pub fn run(ctx: &Context, game: Option<&str>, _skip_artwork: bool) -> Result<ExitCode, Error> {
    let slug = match game {
        Some(s) if !s.trim().is_empty() => s.trim(),
        _ => {
            eprintln!("Error: specify game slug: lewdzone shortcuts <slug>");
            return Ok(ExitCode::Usage);
        }
    };

    let opts = ShortcutOptions {
        desktop: true,
        start_menu: true,
    };

    match create(ctx, slug, opts) {
        Ok(paths) => {
            for p in &paths {
                println!("Created shortcut: {}", p.display());
            }
            Ok(ExitCode::Ok)
        }
        Err(e) => {
            eprintln!("Failed to create shortcuts for {slug}: {e}");
            Ok(ExitCode::Runtime)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_filename_replaces_illegal_characters() {
        assert_eq!(sanitize_filename("Game: Episode 1?"), "Game_ Episode 1_");
        assert_eq!(
            sanitize_filename("Vore/Adventure\\Quest"),
            "Vore_Adventure_Quest"
        );
        assert_eq!(sanitize_filename("Clean Title"), "Clean Title");
    }
}
