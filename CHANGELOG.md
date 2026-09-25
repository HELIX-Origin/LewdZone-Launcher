# LewdZone Launcher Changelog

## Unreleased

(No unreleased changes.)

## [v0.4.1](https://github.com/HELIX-Origin/LewdZone-Launcher/releases/tag/v0.4.1)

### Added

### Changed

* Removed the "Open in Browser" button from the Secure Ad-Free Resolver; "Open in App" remains the primary capture path[(dd43e58)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/dd43e5897c0e613da4bf71b1d70e4ac3e46c1313)
* Unified archive auto-extraction across all supported formats and delete archives after successful extraction[(dd43e58)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/dd43e5897c0e613da4bf71b1d70e4ac3e46c1313)
* Replaced the installer header with the main-app frameless traffic-light titlebar[(dd43e58)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/dd43e5897c0e613da4bf71b1d70e4ac3e46c1313)

### Fixed

* Prevented the background scanner from re-extracting archives already handled by active queue jobs[(dd43e58)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/dd43e5897c0e613da4bf71b1d70e4ac3e46c1313)

## [v0.4.0](https://github.com/HELIX-Origin/LewdZone-Launcher/releases/tag/v0.4.0)

### Added

* Added multi-stream download acceleration with up to 4 concurrent HTTP byte-range streams for direct-file hosts[(0d2887c)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/0d2887c0362fd5f1b1cd50e8f409931f82b9eb6f)
* Expanded `DIRECT_STREAM_HOSTS` to include `fileknot`, `pixeldrain`, `mediafire`, and `workupload`[(0d2887c)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/0d2887c0362fd5f1b1cd50e8f409931f82b9eb6f)

### Changed

* Increased single-stream I/O buffers from 128 KB to 512 KB with 1 MB `BufWriter` caching[(0d2887c)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/0d2887c0362fd5f1b1cd50e8f409931f82b9eb6f)

### Fixed

* Eliminated duplicate native title bars on the installer wizard[(0d2887c)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/0d2887c0362fd5f1b1cd50e8f409931f82b9eb6f)
* Silenced PowerShell subprocess console windows during installation and shortcut creation[(0d2887c)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/0d2887c0362fd5f1b1cd50e8f409931f82b9eb6f)
* Launched the installer as a detached process to avoid file-lock contention[(0d2887c)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/0d2887c0362fd5f1b1cd50e8f409931f82b9eb6f)

## [v0.3.3](https://github.com/HELIX-Origin/LewdZone-Launcher/releases/tag/v0.3.3)

### Added

* Added native backend navigation and a desktop Chrome user agent for the resolver webview to bypass bot detection[(27a8b6e)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/27a8b6e890584ae9faaf4a96ad165884c0d78132)
* Added an `isNavigating` loading state in the resolver view[(27a8b6e)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/27a8b6e890584ae9faaf4a96ad165884c0d78132)

### Changed

* Removed eager URL regex queueing; the resolver now waits for the user to click the actual download button[(27a8b6e)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/27a8b6e890584ae9faaf4a96ad165884c0d78132)

### Fixed

* Fixed in-window link capture by converting `target="_blank"` to `target="_self"` and overriding `window.open`[(27a8b6e)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/27a8b6e890584ae9faaf4a96ad165884c0d78132)

## [v0.3.2](https://github.com/HELIX-Origin/LewdZone-Launcher/releases/tag/v0.3.2)

### Added

* Added automatic termination of stale background/tray instances on installer launch[(ab05908)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/ab05908c3812a2edcca098248caf24a4e815ebff)

### Changed

* Made the default main window invisible during installer mode and destroy it in `run_installer()`[(ab05908)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/ab05908c3812a2edcca098248caf24a4e815ebff)
* Deferred app launch until the user explicitly clicks Finish with "Launch after finish" enabled[(ab05908)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/ab05908c3812a2edcca098248caf24a4e815ebff)

### Fixed

* Fixed WebView2 `Chrome_WidgetWin_0` unregistration error by destroying all webview windows before exit[(ab05908)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/ab05908c3812a2edcca098248caf24a4e815ebff)

## [v0.3.1](https://github.com/HELIX-Origin/LewdZone-Launcher/releases/tag/v0.3.1)

### Added

* Added platform-specific installer naming to avoid CI asset collisions[(081803d)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/081803d6df3926e7591747ee3accbe8903065d42)
* Added detection and cleanup of legacy Tauri NSIS install directories[(081803d)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/081803d6df3926e7591747ee3accbe8903065d42)

### Changed

* Enhanced the installer to terminate running `lewdzone` processes before install/uninstall[(081803d)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/081803d6df3926e7591747ee3accbe8903065d42)

### Fixed

* Fixed direct archive URL detection in the Secure Resolver so archive links enqueue immediately[(081803d)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/081803d6df3926e7591747ee3accbe8903065d42)

## [v0.3.0](https://github.com/HELIX-Origin/LewdZone-Launcher/releases/tag/v0.3.0)

### Added

* Added single-instance enforcement via `tauri-plugin-single-instance`[(4745093)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/4745093e04c7b4c96a906211e7680c5ddaf776df)
* Added client cache and smart polling stores for catalog, details, library, favorites, and settings[(4745093)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/4745093e04c7b4c96a906211e7680c5ddaf776df)
* Added a persistent background service for queue execution and archive ingestion[(4745093)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/4745093e04c7b4c96a906211e7680c5ddaf776df)
* Added the in-app Unified Installer / Uninstaller wizard[(4745093)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/4745093e04c7b4c96a906211e7680c5ddaf776df)
* Added child-window archive interception via Tauri `on_download`[(4745093)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/4745093e04c7b4c96a906211e7680c5ddaf776df)
* Added 2-column Store and Library detail layouts with metadata sidebars[(4745093)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/4745093e04c7b4c96a906211e7680c5ddaf776df)
* Added clean story synopsis scraping from `.content-block.main-content`[(4745093)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/4745093e04c7b4c96a906211e7680c5ddaf776df)
* Added clean library title formatting across manifests and UI[(4745093)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/4745093e04c7b4c96a906211e7680c5ddaf776df)
* Added `enqueue_intercept()` and an intercept processing branch to the download queue[(4745093)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/4745093e04c7b4c96a906211e7680c5ddaf776df)

### Changed

* Added a `title` column to `queue_job` for Title Case game names[(4745093)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/4745093e04c7b4c96a906211e7680c5ddaf776df)
* Replaced legacy WiX/NSIS installers with the unified `LewdZone-Setup` wizard[(4745093)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/4745093e04c7b4c96a906211e7680c5ddaf776df)
* Removed all LewdClips video and external clip references[(4745093)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/4745093e04c7b4c96a906211e7680c5ddaf776df)

### Fixed

* Fixed the redirect loop in the go-token resolver window[(4745093)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/4745093e04c7b4c96a906211e7680c5ddaf776df)

## [v0.2.1](https://github.com/HELIX-Origin/LewdZone-Launcher/releases/tag/v0.2.1)

### Added

* Added SteamGridDB API key settings with masked input and SQLite secret storage[(4cb0a36)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/4cb0a362922771d7370d72ccc7d244119e659f95)

### Changed

* Aligned CI packaging with the official Tauri v2 GitHub Actions workflow[(4cb0a36)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/4cb0a362922771d7370d72ccc7d244119e659f95)
* Switched `beforeBuildCommand` to `npm run build` and enforced `shell: true` in the build script[(4cb0a36)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/4cb0a362922771d7370d72ccc7d244119e659f95)

### Fixed

* Fixed `ENOENT` spawn failures on Linux/macOS during Tauri release bundling[(4cb0a36)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/4cb0a362922771d7370d72ccc7d244119e659f95)

## [v0.2.0](https://github.com/HELIX-Origin/LewdZone-Launcher/releases/tag/v0.2.0)

### Added

* Added native per-OS desktop and Start Menu shortcuts[(2511138)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/2511138b4477bdc6e7456c458b6806378d267a94)
* Added gameplay playtime and session tracking via detached background workers[(2511138)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/2511138b4477bdc6e7456c458b6806378d267a94)
* Added library sort controls with playtime badges and last-played timestamps[(2511138)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/2511138b4477bdc6e7456c458b6806378d267a94)
* Added migration `006_game_playtime` for `game_stats`[(2511138)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/2511138b4477bdc6e7456c458b6806378d267a94)
* Added the `lewdzone shortcuts [SLUG]` CLI command[(2511138)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/2511138b4477bdc6e7456c458b6806378d267a94)

### Changed

* Updated `launch.rs` and `library.rs` to record and load playtime statistics[(2511138)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/2511138b4477bdc6e7456c458b6806378d267a94)

### Fixed

## [v0.1.0](https://github.com/HELIX-Origin/LewdZone-Launcher/releases/tag/v0.1.0)

### Added

* Initial Tauri 2 + Rust desktop GUI and native CLI sharing one core[(b290bfc)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/b290bfc765399c941db27fc3679d5cb9e594d056)
* Catalog browsing, search, and filtering for lewdzone.com[(b290bfc)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/b290bfc765399c941db27fc3679d5cb9e594d056)
* Two-step go-link token resolver with sandboxed challenge webview[(b290bfc)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/b290bfc765399c941db27fc3679d5cb9e594d056)
* In-app streaming for direct-file hosts and OS default-handler dispatch for cloud hosts[(b290bfc)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/b290bfc765399c941db27fc3679d5cb9e594d056)
* Host blacklist for defunct/malicious mirrors[(b290bfc)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/b290bfc765399c941db27fc3679d5cb9e594d056)
* 7-Zip CLI multi-format extraction with real-time progress[(b290bfc)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/b290bfc765399c941db27fc3679d5cb9e594d056)
* Flat library layout with `app.json` manifests and game scanner/launcher[(b290bfc)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/b290bfc765399c941db27fc3679d5cb9e594d056)
* Persistent SQLite download queue[(b290bfc)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/b290bfc765399c941db27fc3679d5cb9e594d056)
* System tray integration with quick-access context menu[(b290bfc)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/b290bfc765399c941db27fc3679d5cb9e594d056)
* Dynamic Nord/Dracula/Material theming[(b290bfc)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/b290bfc765399c941db27fc3679d5cb9e594d056)
* External metadata and artwork enrichment from SteamGridDB, VNDB, IGDB, itch.io, Steam, and IndieDB[(b290bfc)](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/b290bfc765399c941db27fc3679d5cb9e594d056)

### Changed

### Fixed

---

| Version | Title | Description | Status |
| :---: | :---: | :---: | :---: |
| v0.4.1 | Resolver cleanup, unified extraction, installer titlebar | Removed browser resolver button; auto-extract all archive formats; matching installer titlebar | ✅ |
| v0.4.0 | Accelerated downloads, installer UI fixes | Multi-stream downloads; expanded direct-stream hosts; fixed installer titlebar and console flashes | ✅ |
| v0.3.3 | Resolver navigation fixes | Desktop user agent; removed premature queueing; in-window capture; loading state | ✅ |
| v0.3.2 | Installer process isolation | Hidden main window during setup; stale-instance termination; deferred launch; clean WebView2 teardown | ✅ |
| v0.3.1 | Installer naming and capture fixes | Platform-specific installer names; legacy NSIS cleanup; process termination; direct-archive capture | ✅ |
| v0.3.0 | Unified installer wizard and UI refresh | Single-instance enforcement; background service; installer wizard; archive interception; 2-column layouts | ✅ |
| v0.2.1 | CI packaging and SteamGridDB settings | Tauri v2 CI alignment; SteamGridDB API key persistence | ✅ |
| v0.2.0 | Shortcuts, playtime tracking, and CI packaging | Native shortcuts; session tracking; library sorting; multi-platform CI packaging | ✅ |
| v0.1.0 | Initial release | First public release with catalog, resolver, downloads, extraction, library, and theming | ✅ |

---

**Last updated:** 2026-09-25
