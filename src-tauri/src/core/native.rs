//! `native` — OS default-handler pass-through.
//!
//! Hosts that are not direct-file streams (see `download::DIRECT_STREAM_HOSTS`)
//! hand their resolved URL to the operating system's default handler — the
//! installed cloud app (MEGA, Google Drive, Dropbox, …) or the browser — with
//! zero configuration. The URL still goes through the resolver allowlist
//! (Rule 10.2) before it can be opened here.
//!
//! The actual "open URL with the default handler" call lives in a per-OS
//! submodule (`platform::open_url`) so Windows, macOS, and Linux each have an
//! explicit implementation.

use crate::core::Error;

/// Human-readable app name for a cloud host slug.
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
    fn app_names_are_human_readable() {
        assert_eq!(app_name("mega"), "MEGA");
        assert_eq!(app_name("pixeldrain"), "pixeldrain");
        assert_eq!(app_name("fileknot"), "native cloud app");
    }
}
