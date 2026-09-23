//! `native` — native cloud-app pass-through (source-priority setting).
//!
//! Cloud hosts with their own desktop client (Google Drive, Dropbox, MediaFire,
//! MEGA) can receive downloads without a download manager: when the
//! `native-cloud` setting is on and the chosen entry resolves to one of these
//! hosts, the resolved URL is handed to the operating system's default handler
//! instead of a DM adapter. The URL still goes through the resolver allowlist
//! (Rule 10.2) before it can be opened here.
//!
//! The actual "open URL with the default handler" call lives in a per-OS
//! submodule (`platform::open_url`) so Windows, macOS, and Linux each have an
//! explicit implementation.

use crate::core::Error;

/// Host slugs that have a native desktop app/browser handler.
pub const NATIVE_CLOUD_HOSTS: &[&str] = &["google", "dropbox", "mediafire", "mega", "pixeldrain"];

/// Human-readable app name for a native-cloud host slug.
pub fn app_name(host: &str) -> &'static str {
    match host {
        "google" => "Google Drive",
        "dropbox" => "Dropbox",
        "mediafire" => "MediaFire",
        "mega" => "MEGA",
        "pixeldrain" => "pixeldrain",
        _ => "native cloud app",
    }
}

/// Whether `host` should be passed to its native desktop app when `native-cloud`
/// is enabled.
pub fn is_native_cloud_host(host: &str) -> bool {
    NATIVE_CLOUD_HOSTS.contains(&host)
}

/// Whether the `native-cloud` setting is enabled for a specific host.
pub fn native_cloud_enabled(settings: &crate::core::settings::Settings, host: &str) -> bool {
    settings.native_cloud == Some(true) && is_native_cloud_host(host)
}

/// Open `url` with the OS default handler (the native cloud app if installed),
/// detached and without waiting. Dispatches to the per-OS implementation.
pub fn open_url(url: &str) -> Result<(), Error> {
    platform::open_url(url)
}

// Per-OS default-handler implementations. Each platform file owns the full
// command + spawn flags for its OS so none of the three is an afterthought.
#[cfg(target_os = "windows")]
mod platform {
    use super::Error;
    use std::os::windows::process::CommandExt as _;
    pub(super) fn open_url(url: &str) -> Result<(), Error> {
        let mut proc = std::process::Command::new("rundll32");
        proc.args(["url.dll,FileProtocolHandler", url]);
        proc.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
        proc.spawn()
            .map(|_| ())
            .map_err(|e| Error::Runtime(format!("failed to open '{url}': {e}")))
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::Error;
    use std::os::unix::process::CommandExt as _;
    pub(super) fn open_url(url: &str) -> Result<(), Error> {
        let mut proc = std::process::Command::new("open");
        proc.arg(url);
        proc.process_group(0);
        proc.spawn()
            .map(|_| ())
            .map_err(|e| Error::Runtime(format!("failed to open '{url}': {e}")))
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
mod platform {
    use super::Error;
    use std::os::unix::process::CommandExt as _;
    pub(super) fn open_url(url: &str) -> Result<(), Error> {
        let mut proc = std::process::Command::new("xdg-open");
        proc.arg(url);
        proc.process_group(0);
        proc.spawn()
            .map(|_| ())
            .map_err(|e| Error::Runtime(format!("failed to open '{url}': {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_cloud_hosts_are_the_cloud_apps() {
        assert!(is_native_cloud_host("google"));
        assert!(is_native_cloud_host("dropbox"));
        assert!(is_native_cloud_host("mediafire"));
        assert!(is_native_cloud_host("mega"));
        assert!(is_native_cloud_host("pixeldrain"));
        assert!(!is_native_cloud_host("fileknot"));
        assert!(!is_native_cloud_host("transfaze"));
    }

    #[test]
    fn app_names_are_human_readable() {
        assert_eq!(app_name("mega"), "MEGA");
        assert_eq!(app_name("pixeldrain"), "pixeldrain");
        assert_eq!(app_name("fileknot"), "native cloud app");
    }

    #[test]
    fn native_cloud_enabled_requires_both_setting_and_host() {
        let off = crate::core::settings::Settings::default();
        assert!(!native_cloud_enabled(&off, "mega"));
        let on = crate::core::settings::Settings {
            native_cloud: Some(true),
            ..Default::default()
        };
        assert!(native_cloud_enabled(&on, "mega"));
        assert!(!native_cloud_enabled(&on, "fileknot"));
    }
}
