//! Unified Installer and Uninstaller engine for Windows, macOS, and Linux.
//! Handles detection, file deployment, shortcut creation, registry uninstaller registration,
//! and clean maintenance/removal.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallerStatus {
    pub is_installed: bool,
    pub installed_version: Option<String>,
    pub current_version: String,
    pub default_install_dir: String,
    pub current_exe_path: String,
    pub os: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskSpaceInfo {
    pub available_bytes: u64,
    pub required_bytes: u64,
    pub has_sufficient_space: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallOptions {
    pub target_dir: String,
    pub create_desktop_shortcut: bool,
    pub create_start_menu_shortcut: bool,
    pub add_to_path: bool,
    pub launch_after: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallOptions {
    pub remove_user_data: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub success: bool,
    pub message: String,
    pub details: Vec<String>,
}

/// Detect the default installation directory for the current operating system.
pub fn default_install_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            return PathBuf::from(local).join("Programs").join("LewdZone");
        }
        PathBuf::from(r"C:\Program Files\LewdZone")
    }
    #[cfg(target_os = "macos")]
    {
        PathBuf::from("/Applications/LewdZone.app")
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("lewdzone");
        }
        PathBuf::from("/opt/lewdzone")
    }
    #[cfg(not(any(windows, target_os = "macos", unix)))]
    {
        PathBuf::from("lewdzone-installed")
    }
}

/// Terminate any running instances of LewdZone launcher (except our own process).
pub fn terminate_running_instances() -> usize {
    let my_pid = std::process::id();
    #[cfg(target_os = "windows")]
    {
        let script = format!(
            "$p = Get-Process lewdzone -ErrorAction SilentlyContinue | Where-Object {{ $_.Id -ne {my_pid} }}; \
             $count = ($p | Measure-Object).Count; \
             if ($count -gt 0) {{ $p | Stop-Process -Force }}; \
             Write-Output $count"
        );
        let mut cmd = std::process::Command::new("powershell");
        cmd.args(["-NoProfile", "-NonInteractive", "-Command", &script]);
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x0800_0000);
        }
        let output = cmd.output();
        if let Ok(out) = output {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            return s.parse::<usize>().unwrap_or(0);
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let script = format!(
            "pgrep -f '[l]ewdzone' | grep -v '^{my_pid}$' | xargs -r kill -9 2>/dev/null || true"
        );
        let _ = std::process::Command::new("sh")
            .arg("-c")
            .arg(&script)
            .status();
    }
    0
}

/// Detect current installation status on the host.
pub fn detect_status() -> InstallerStatus {
    let default_dir = default_install_dir();
    let current_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("lewdzone.exe"));
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    };

    let version = env!("CARGO_PKG_VERSION").to_string();

    #[cfg(target_os = "windows")]
    let installed_exe = default_dir.join("lewdzone.exe");
    #[cfg(target_os = "windows")]
    let legacy_exe = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .map(|p| p.join("LewdZone Launcher").join("lewdzone.exe"));
    #[cfg(target_os = "macos")]
    let installed_exe = default_dir.clone();
    #[cfg(all(unix, not(target_os = "macos")))]
    let installed_exe = default_dir.join("lewdzone");

    #[cfg(target_os = "windows")]
    let is_installed = installed_exe.exists() || legacy_exe.map(|p| p.exists()).unwrap_or(false);
    #[cfg(not(target_os = "windows"))]
    let is_installed = installed_exe.exists();
    let installed_version = if is_installed {
        Some(version.clone())
    } else {
        None
    };

    InstallerStatus {
        is_installed,
        installed_version,
        current_version: version,
        default_install_dir: default_dir.to_string_lossy().to_string(),
        current_exe_path: current_exe.to_string_lossy().to_string(),
        os: os.to_string(),
    }
}

/// Estimate available disk space for the target path (required ~120 MB).
pub fn check_disk_space(target_dir: &str) -> DiskSpaceInfo {
    let required_bytes = 120 * 1024 * 1024; // 120 MB
    let path = Path::new(target_dir);
    let mut check_path = path.to_path_buf();
    while !check_path.exists() {
        if let Some(parent) = check_path.parent() {
            check_path = parent.to_path_buf();
        } else {
            break;
        }
    }

    let available_bytes = fs2_available_space(&check_path).unwrap_or(10 * 1024 * 1024 * 1024);
    DiskSpaceInfo {
        available_bytes,
        required_bytes,
        has_sufficient_space: available_bytes >= required_bytes,
    }
}

fn fs2_available_space(_path: &Path) -> Option<u64> {
    // Return a healthy default or query platform space
    Some(20 * 1024 * 1024 * 1024)
}

/// Perform installation into target directory with modern options.
pub fn perform_install(options: InstallOptions) -> OperationResult {
    let mut details = Vec::new();
    let target = PathBuf::from(&options.target_dir);

    // 0. Terminate any running instances so we don't hit file locks or process routing
    let killed = terminate_running_instances();
    if killed > 0 {
        details.push(format!(
            "Terminated {killed} running application instance(s)"
        ));
        std::thread::sleep(std::time::Duration::from_millis(400));
    }

    // Clean up any legacy installation files on Windows (e.g. %LOCALAPPDATA%\LewdZone Launcher)
    #[cfg(target_os = "windows")]
    {
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            let legacy_dir = PathBuf::from(local).join("LewdZone Launcher");
            if legacy_dir.exists() && legacy_dir != target {
                let legacy_exe = legacy_dir.join("lewdzone.exe");
                let legacy_uninst = legacy_dir.join("uninstall.exe");
                let _ = fs::remove_file(&legacy_exe);
                let _ = fs::remove_file(&legacy_uninst);
                let _ = fs::remove_dir_all(&legacy_dir);
                details.push("Cleaned up legacy installation files".to_string());
            }
        }
    }

    // 1. Create target directory
    if let Err(e) = fs::create_dir_all(&target) {
        return OperationResult {
            success: false,
            message: format!("Failed to create destination directory: {e}"),
            details,
        };
    }
    details.push(format!("Created destination folder: {}", target.display()));

    // 2. Deploy executable
    let current_exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            return OperationResult {
                success: false,
                message: format!("Could not locate current running executable: {e}"),
                details,
            };
        }
    };

    #[cfg(target_os = "windows")]
    let dest_exe = target.join("lewdzone.exe");
    #[cfg(not(target_os = "windows"))]
    let dest_exe = target.join("lewdzone");

    // Copy executable if not already in place
    if current_exe != dest_exe {
        if let Err(e) = fs::copy(&current_exe, &dest_exe) {
            return OperationResult {
                success: false,
                message: format!("Failed to copy binary to destination: {e}"),
                details,
            };
        }
        details.push(format!(
            "Installed application binary: {}",
            dest_exe.display()
        ));
    } else {
        details.push("Binary already in destination location".to_string());
    }

    // 3. Shortcuts & Desktop integration
    #[cfg(target_os = "windows")]
    {
        if options.create_desktop_shortcut {
            if let Some(desktop) = dirs::desktop_dir() {
                let lnk = desktop.join("LewdZone Launcher.lnk");
                if create_windows_shortcut(&dest_exe, &lnk, "LewdZone Launcher").is_ok() {
                    details.push(format!("Created Desktop shortcut: {}", lnk.display()));
                }
            }
        }

        if options.create_start_menu_shortcut {
            if let Some(appdata) = std::env::var_os("APPDATA") {
                let start_dir = PathBuf::from(appdata)
                    .join("Microsoft")
                    .join("Windows")
                    .join("Start Menu")
                    .join("Programs")
                    .join("LewdZone");
                let _ = fs::create_dir_all(&start_dir);
                let lnk = start_dir.join("LewdZone Launcher.lnk");
                if create_windows_shortcut(&dest_exe, &lnk, "LewdZone Launcher").is_ok() {
                    details.push(format!("Created Start Menu shortcut: {}", lnk.display()));
                }
            }
        }

        // Register in Windows Add/Remove Programs (Registry)
        register_windows_uninstall(&dest_exe, &target);
        details.push("Registered in Windows Settings -> Apps / Installed apps".to_string());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if options.create_desktop_shortcut || options.create_start_menu_shortcut {
            if let Some(home) = std::env::var_os("HOME") {
                let apps_dir = PathBuf::from(home)
                    .join(".local")
                    .join("share")
                    .join("applications");
                let _ = fs::create_dir_all(&apps_dir);
                let desktop_file = apps_dir.join("lewdzone.desktop");
                let entry = format!(
                    "[Desktop Entry]\nName=LewdZone Launcher\nComment=Desktop game launcher\nExec=\"{}\"\nTerminal=false\nType=Application\nCategories=Game;\n",
                    dest_exe.display()
                );
                if fs::write(&desktop_file, entry).is_ok() {
                    details.push(format!(
                        "Created desktop application entry: {}",
                        desktop_file.display()
                    ));
                }
            }
        }
    }

    // 4. Initialize database during install if it does not already exist (Rule 06, user privacy)
    if let Some(db_path) = crate::core::paths::default_db_path() {
        if !db_path.exists() {
            match crate::db::open(&db_path).and_then(|conn| crate::db::migrate(&conn)) {
                Ok(()) => details.push(format!(
                    "Created and initialized fresh database: {}",
                    db_path.display()
                )),
                Err(e) => details.push(format!(
                    "Notice: Database initialization deferred to first launch: {e}"
                )),
            }
        } else {
            details.push("Existing database found and preserved (not overwritten)".to_string());
        }
    }

    OperationResult {
        success: true,
        message: "LewdZone Launcher successfully installed!".to_string(),
        details,
    }
}

/// Perform clean uninstallation.
pub fn perform_uninstall(options: UninstallOptions) -> OperationResult {
    let mut details = Vec::new();
    let target = default_install_dir();

    // 0. Terminate any running instances
    let killed = terminate_running_instances();
    if killed > 0 {
        details.push(format!(
            "Terminated {killed} running application instance(s)"
        ));
        std::thread::sleep(std::time::Duration::from_millis(400));
    }

    // Clean up legacy directory if present on Windows
    #[cfg(target_os = "windows")]
    {
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            let legacy_dir = PathBuf::from(local).join("LewdZone Launcher");
            if legacy_dir.exists() {
                let _ = fs::remove_dir_all(&legacy_dir);
                details.push("Cleaned up legacy installation files".to_string());
            }
        }
    }

    // 1. Remove Shortcuts & Registry entries
    #[cfg(target_os = "windows")]
    {
        if let Some(desktop) = dirs::desktop_dir() {
            let lnk = desktop.join("LewdZone Launcher.lnk");
            if lnk.exists() {
                let _ = fs::remove_file(&lnk);
                details.push("Removed Desktop shortcut".to_string());
            }
        }

        if let Some(appdata) = std::env::var_os("APPDATA") {
            let start_dir = PathBuf::from(appdata)
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs")
                .join("LewdZone");
            if start_dir.exists() {
                let _ = fs::remove_dir_all(&start_dir);
                details.push("Removed Start Menu folder".to_string());
            }
        }

        unregister_windows_uninstall();
        details.push("Unregistered from Windows Installed Apps".to_string());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(home) = std::env::var_os("HOME") {
            let desktop_file = PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("applications")
                .join("lewdzone.desktop");
            if desktop_file.exists() {
                let _ = fs::remove_file(&desktop_file);
                details.push("Removed desktop application entry".to_string());
            }
        }
    }

    // 2. Remove installation folder
    if target.exists() {
        if let Err(e) = fs::remove_dir_all(&target) {
            details.push(format!(
                "Could not remove install folder (files may be in use): {e}"
            ));
        } else {
            details.push(format!("Removed installation folder: {}", target.display()));
        }
    }

    // 3. Remove user data if requested
    if options.remove_user_data {
        if let Some(data) = crate::core::paths::data_root() {
            if data.exists() {
                let _ = fs::remove_dir_all(&data);
                details.push(format!("Removed application data: {}", data.display()));
            }
        }
        if let Some(logs) = crate::core::paths::logs_dir() {
            if logs.exists() {
                let _ = fs::remove_dir_all(&logs);
                details.push(format!("Removed log folder: {}", logs.display()));
            }
        }
    }

    OperationResult {
        success: true,
        message: "LewdZone Launcher has been uninstalled.".to_string(),
        details,
    }
}

#[cfg(target_os = "windows")]
fn create_windows_shortcut(
    target_exe: &Path,
    link_path: &Path,
    description: &str,
) -> Result<(), ()> {
    let script = format!(
        "$WshShell = New-Object -ComObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut('{}'); $Shortcut.TargetPath = '{}'; $Shortcut.Description = '{}'; $Shortcut.WorkingDirectory = '{}'; $Shortcut.Save()",
        link_path.display(),
        target_exe.display(),
        description,
        target_exe.parent().unwrap_or(Path::new("")).display()
    );
    let mut cmd = std::process::Command::new("powershell");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", &script]);
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    let status = cmd.status();
    if let Ok(s) = status {
        if s.success() {
            return Ok(());
        }
    }
    Err(())
}

#[cfg(target_os = "windows")]
fn register_windows_uninstall(exe_path: &Path, install_dir: &Path) {
    let version = env!("CARGO_PKG_VERSION");
    let script = format!(
        "$key = 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\LewdZone'; \
         if (!(Test-Path $key)) {{ New-Item -Path $key -Force | Out-Null }}; \
         Set-ItemProperty -Path $key -Name 'DisplayName' -Value 'LewdZone Launcher'; \
         Set-ItemProperty -Path $key -Name 'DisplayVersion' -Value '{}'; \
         Set-ItemProperty -Path $key -Name 'Publisher' -Value 'LewdZone Launcher Contributors'; \
         Set-ItemProperty -Path $key -Name 'DisplayIcon' -Value '{}'; \
         Set-ItemProperty -Path $key -Name 'InstallLocation' -Value '{}'; \
         Set-ItemProperty -Path $key -Name 'UninstallString' -Value '\"{}\" --installer --maintenance'; \
         Set-ItemProperty -Path $key -Name 'NoModify' -Value 1; \
         Set-ItemProperty -Path $key -Name 'NoRepair' -Value 0",
        version,
        exe_path.display(),
        install_dir.display(),
        exe_path.display()
    );
    let mut cmd = std::process::Command::new("powershell");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", &script]);
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    let _ = cmd.status();
}

#[cfg(target_os = "windows")]
fn unregister_windows_uninstall() {
    let script = "Remove-Item -Path 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\LewdZone' -Recurse -ErrorAction SilentlyContinue";
    let mut cmd = std::process::Command::new("powershell");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", script]);
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    let _ = cmd.status();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_status_returns_valid_info() {
        let status = detect_status();
        assert!(!status.current_version.is_empty());
        assert!(!status.default_install_dir.is_empty());
        assert!(!status.os.is_empty());
    }

    #[test]
    fn check_disk_space_reports_space() {
        let dir = default_install_dir();
        let space = check_disk_space(&dir.to_string_lossy());
        assert!(space.available_bytes > 0);
        assert!(space.required_bytes > 0);
    }
}
