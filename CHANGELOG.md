# 📜 LewdZone Launcher Changelog

Historical record of every change to the repository.

## Unreleased

(No unreleased changes.)

## [v0.5.0](https://github.com/HELIX-Origin/LewdZone-Launcher/releases/tag/v0.5.0)

### ✨ Added

* **Titlebar Search**: Moved the game search box into the custom frameless titlebar, centered above the window
    * Search is context-aware: it filters the Store catalog (`/store`), installed games (`/library`), or favorites (`/favorites`) based on the active view
    * Placeholder text updates per scope ("Search store…", "Search library…", "Search favorites…")
    * Library and Favorites filter client-side by title without extra round trips; Store keeps its server-backed catalog query
* **Resolver Ad Blocking on Host Pages**: Extended the resolver's ad/popup protection to the external download-host pages it navigates to
    * Hides or removes common ad containers, ad/tracker script tags, and ad-network iframes via a `MutationObserver` that keeps enforcing as pages load dynamically
    * Blocks `window.open`, `alert`, `confirm`, and `prompt` on host pages, and rewrites `_blank` links to `_self`
    * Injected both at webview creation (`initialization_script`) and after every in-app navigation via `resolver_navigate_in_app`

### 🔄 Changed

* **Store Page**: Removed the now-redundant in-page search input; the titlebar search is the single search entry point
* **Version Metadata**: Bumped to `0.5.0` across `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, and `src-tauri/tauri.conf.json`

### 🐛 Fixed

* **Titlebar Search Scope**: Replaced an invalid IIFE inside `$derived` that broke `svelte-check` on the layout route

## [v0.4.1](https://github.com/HELIX-Origin/LewdZone-Launcher/releases/tag/v0.4.1)

### ✨ Added

### 🔄 Changed

* **Resolver UI**: Removed the "Open in Browser" button from the Secure Ad-Free Resolver; "Open in App" remains the primary capture path
    * Simplifies the resolver to a single in-app capture path and eliminates confusion between browser and app dispatch
* **Archive Extraction**: Unified archive auto-extraction across all supported formats
    * `stream_target` and `stream_target_accelerated` now auto-extract `.zip`, `.7z`, `.rar`, `.tar.gz`, `.tar.bz2`, `.tar.xz`, `.tgz`, `.tbz2`, `.txz`, and SFX `.exe`
    * Successfully extracted archives are deleted automatically, eliminating leftover files in the download directory
* **Installer Titlebar**: Replaced the installer header with the main-app frameless traffic-light titlebar
    * Includes theme-aware glass styling, drag-to-move behavior, and correct macOS/Windows control ordering

### 🐛 Fixed

* **Background Scanner**: Prevented the background scanner from re-extracting archives already handled by active queue jobs
    * `ingest_completed_archives` now skips archives modified in the last 60 seconds
    * Also skips any archive whose path or target slug is already being processed by an active intercept/download/extract queue job

## [v0.4.0](https://github.com/HELIX-Origin/LewdZone-Launcher/releases/tag/v0.4.0)

### ✨ Added

* **Multi-Stream Download Acceleration**: Implemented parallel segmented chunk downloading for in-app archive downloads
    * Supports up to 4 concurrent HTTP byte-range (`Range: bytes=start-end`) streams
    * Writes directly into preallocated archive files via OS positioned writes (`seek_write` on Windows, `write_all_at` on Unix)
    * Multiplies throughput by 3x–6x on remote direct-file hosts
* **Direct-Stream Host Expansion**: Synchronized `DIRECT_STREAM_HOSTS` across the download core and background service
    * Added `fileknot`, `pixeldrain`, `mediafire`, and `workupload` to the in-app streaming allowlist

### 🔄 Changed

* **Stream Buffer**: Expanded streaming I/O buffers from 128 KB to 512 KB with 1 MB `BufWriter` caching
    * Reduces syscall overhead and improves drive write performance for non-range streams and small archives

### 🐛 Fixed

* **Installer Titlebars**: Eliminated duplicate native title bars on the installer wizard
    * Configured `decorations: false` on the setup webview window in both `run_installer` and `open_installer_window`
    * Ensures only the custom HTML frameless titlebar renders without an overlapping OS native titlebar
* **Console Windows**: Silenced PowerShell subprocess console windows during installation and shortcut creation
    * Added `CREATE_NO_WINDOW` (`0x08000000`) flags to all PowerShell invocations in `core/installer.rs` and `core/shortcuts.rs`
    * Prevents flashing `conhost.exe` popups during install, shortcut creation, and registry modifications
* **Installer Launch**: Launched the installer as an independent detached process to avoid file-lock contention
    * Spawns with `--installer` / `--maintenance` and cleanly exits the main launcher application

## [v0.3.3](https://github.com/HELIX-Origin/LewdZone-Launcher/releases/tag/v0.3.3)

### ✨ Added

* **Resolver Navigation**: Added native backend navigation and a desktop Chrome user agent for the resolver webview
    * Bypasses embedded WebView2 bot/Cloudflare detection on file hosts (Pixeldrain, Mediafire, Mega) that previously rendered a blank white page
    * Enables cross-origin navigation inside the sandboxed resolver
* **Loading State**: Added an `isNavigating` loading state in the resolver view
    * Informs the user that the host download page is loading in-app

### 🔄 Changed

* **Queue Trigger**: Removed eager URL regex queueing from `openInApp()`
    * The resolver now waits for the user to view the host page and click the actual download button
    * Eliminates premature download queueing for landing-page URLs that merely contain filenames

### 🐛 Fixed

* **Link Capture**: Fixed in-window link and popup capture inside the resolver
    * Injected initialization script converts `target="_blank"` anchor clicks to `target="_self"`
    * Overrides `window.open` to navigate inside the same window so download clicks are caught by the native `on_download` hook

## [v0.3.2](https://github.com/HELIX-Origin/LewdZone-Launcher/releases/tag/v0.3.2)

### ✨ Added

* **Stale Instance Termination**: Added automatic termination of stale background/tray instances on installer launch
    * Prevents older or lingering application instances from intercepting single-instance focus or showing obsolete UI during setup

### 🔄 Changed

* **Installer Window Isolation**: Made the default main window invisible during installer mode
    * The `"main"` window is configured to `visible: false` and explicitly destroyed in `run_installer()`
    * Prevents the main game launcher window and background services from spawning alongside the installer wizard
* **Deferred Launch**: Deferred application launch until the user explicitly confirms on setup completion
    * Removed premature executable spawning from `perform_install()`
    * The installed launcher now only launches via `installer_launch_app` when the user clicks "Finish" with "Launch after finish" enabled

### 🐛 Fixed

* **WebView2 Teardown**: Fixed `Failed to unregister class Chrome_WidgetWin_0. Error = 1412`
    * Explicitly destroys all open webview windows before `app.exit(0)`
    * Also destroys webview windows when closing the installer and resolver windows

## [v0.3.1](https://github.com/HELIX-Origin/LewdZone-Launcher/releases/tag/v0.3.1)

### ✨ Added

* **Platform Installer Names**: Added platform-specific installer naming to avoid CI asset collisions
    * Generates clearly distinguished binaries such as `LewdZone-Setup-v0.3.1-windows-x64.exe`, `LewdZone-Setup-v0.3.1-linux-x64`, and `LewdZone-Setup-v0.3.1-macos-arm64`
    * Makes platform targets immediately clear to users and prevents collisions between Linux and macOS artifacts
* **Legacy Cleanup**: Added detection and cleanup of legacy Tauri NSIS install directories
    * Removes `%LOCALAPPDATA%\LewdZone Launcher\` to prevent stale executable versions from persisting alongside new unified installs

### 🔄 Changed

* **Process Termination**: Enhanced the installer to terminate running `lewdzone` processes before install/uninstall
    * Resolves file locks (`Access is denied`) during deployment
    * Prevents old background/tray instances from intercepting single-instance focus on update

### 🐛 Fixed

* **Direct Archive Capture**: Fixed direct archive URL detection in the Secure Resolver child window
    * Detects direct archive URLs (`.zip`, `.7z`, `.rar`, `.001`, `.part1.rar`, etc.) and immediately enqueues them to the download worker
    * Seamlessly closes the resolver window after interception
    * Expanded archive format detection in Tauri's webview download interceptor

## [v0.3.0](https://github.com/HELIX-Origin/LewdZone-Launcher/releases/tag/v0.3.0)

### ✨ Added

* **Single-Instance Enforcement**: Integrated `tauri-plugin-single-instance`
    * Launching duplicate instances automatically focuses the existing running application window
* **Client Cache & Smart Polling**: Added centralized Svelte stores (`src/lib/stores/clientCache.ts`)
    * Covers catalog, details, library, favorites, and settings
    * Smart conditional polling eliminates redundant page reloads and network queries while keeping download progress responsive
* **Persistent Background Service**: Added a dedicated background worker (`src-tauri/src/core/service.rs`)
    * Coordinates task queue execution, external host URL resolution, streaming downloads, and automatic ingestion of completed archives
* **Unified Installer Wizard**: Added an in-app setup and maintenance wizard (`src/routes/installer/+page.svelte`)
    * Replaced legacy WiX/NSIS installers with a single lightweight binary
    * Handles clean installs, component selection, start menu shortcuts, uninstallation, and maintenance mode
* **Child-Window Archive Interception**: Intercepts archive download requests clicked within the redirect resolver window
    * Uses Tauri 2's `on_download` hook for `.zip`, `.7z`, `.rar`, `.exe` SFX, and `.tar.gz`
    * Cancels the OS browser dialog, captures the direct URL, and streams the archive in-app with live byte progress
* **2-Column Layouts**: Restructured Store (`/store/[slug]`) and Library (`/library/[slug]`) detail views
    * Modern `1fr 320px` layout with a sticky metadata card
    * Displays game status badges, developer, engine, size on disk, rating, censorship, platform badges, and direct external links
* **Clean Story Synopsis**: Upgraded the scraper to parse clean plot and synopsis paragraphs
    * Sources from `.content-block.main-content` and eliminates SEO promotional boilerplate
* **Clean Library Titles**: Sanitized game titles across local folder scanning, manifest generation, and UI display
    * Strips archive extensions, version suffixes, and platform tags from titles
* **Canonical Release Descriptors**: Added a dedicated `title` column to `queue_job` via migration `008_queue_job_title`
    * Ensures Title Case game names across the Downloads page
    * Resolves chapter/version separation such as `Version 1.01 Chapter 1-4` across the resolver, queue, and filesystem archives
* **Privacy & Secure Database**: Ensured the SQLite database (`lewdzone.db`) is strictly local and never bundled or committed
    * Installer checks and initializes a fresh database schema on install while preserving existing user data
    * Keeps tokens, secrets, and library progress private in `%APPDATA%\lewdzone\`
* **Content Enrichment**: Added multi-source media enrichment and a shared carousel supporting up to 10 images
* **Installed Game Details**: Added a dedicated installed-game details page and streamlined library cards

### 🔄 Changed

* **Installer Replacement**: Replaced legacy WiX/NSIS installers with the unified `LewdZone-Setup` wizard
* **LewdClips Removal**: Completely removed all third-party video and external clip links and fixture references from the codebase
* **Queue Intercept**: Added `enqueue_intercept()` and an intercept processing branch to the download queue
    * Enables direct-URL streaming and extraction dispatch without going through go-link resolvers

### 🐛 Fixed

* **Resolver Redirect**: Fixed the redirect loop in the go-token resolver window
    * Added an "Open in App" path alongside the existing browser path so hosting landing pages can be navigated directly in-app

## [v0.2.1](https://github.com/HELIX-Origin/LewdZone-Launcher/releases/tag/v0.2.1)

### ✨ Added

* **SteamGridDB Settings**: Added a Content Providers section to the Settings page
    * Users can input their SteamGridDB API key with masked password display
    * Keys are securely stored in the SQLite `secret` table and never written to `config.json`
    * Displays live *Key saved* / *Not configured* status indicators with Save and Clear controls

### 🔄 Changed

* **CI Alignment**: Aligned CI packaging with the official Tauri v2 GitHub Actions workflow
    * Updated `.github/workflows/package.yml` to install dual macOS targets (`aarch64-apple-darwin` and `x86_64-apple-darwin`)
    * Set `node-version: lts/*` and provided native system libraries (`libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf`)
* **Build Script**: Switched `beforeBuildCommand` to `npm run build` and enforced `shell: true` in `scripts/build-with-log.mjs`
* **Native Artifact Dropping**: Replaced the legacy `build/` root directory and `copy-installers.mjs` script
    * Collects native bundle artifacts directly from `src-tauri/target/release/bundle/`
    * Cleaned `.gitignore` and `package.json` scripts accordingly
* **Workflow Dispatch Safety**: Scoped `tagName` in `tauri-action` to `v*` tag push events
    * Allows manual packaging test dispatches to upload artifacts cleanly without attempting to publish duplicate release tags

### 🐛 Fixed

* **CI Packaging Pipeline**: Fixed `ENOENT` spawn failures on Linux and macOS during Tauri release bundling

## [v0.2.0](https://github.com/HELIX-Origin/LewdZone-Launcher/releases/tag/v0.2.0)

### ✨ Added

* **Native Shortcuts**: Generate native per-OS desktop and Start Menu shortcuts
    * Windows `.lnk` shortcuts point directly to the game binary with the executable's embedded icon index
    * Linux uses FreeDesktop `.desktop` entries
    * macOS uses `.command` aliases
* **Playtime Tracking**: Monitor game child processes in detached background worker threads
    * Calculates elapsed playtime upon exit and persists metrics to SQLite (`game_stats`)
* **Library Sorting**: Sort installed games by A–Z, Recently Played, Most Played, or Recently Installed
    * Includes formatted playtime badges and last-played timestamps
* **Automated Packaging**: Added a multi-OS CI/CD packaging workflow (`.github/workflows/package.yml`)
    * Builds Windows, macOS, and Linux release artifacts
* **Playtime Migration**: Added migration `006_game_playtime` creating the `game_stats` table
    * Tracks `playtime_seconds`, `play_count`, and `last_played_at`
* **Shortcuts CLI**: Added the `lewdzone shortcuts [SLUG]` command

### 🔄 Changed

* **Playtime Persistence**: Updated `launch.rs` and `library.rs` to record and load playtime statistics
* **UI Additions**: Added shortcut button on game cards, shortcut toast alert, sort dropdown, and playtime badges

### 🐛 Fixed

## [v0.1.0](https://github.com/HELIX-Origin/LewdZone-Launcher/releases/tag/v0.1.0)

### ✨ Added

* **Unified GUI & CLI**: Initial Tauri 2 + Rust desktop GUI and native CLI sharing one core
    * `lewdzone <cmd> --json` provides 100% feature parity with the desktop app
* **Catalog Browsing**: Built-in scraper for lewdzone.com catalog
    * Supports tag/genre filtering, engine selectors (Ren'Py, RPG Maker, Unity, HTML), release status, and pagination
* **Token Resolver**: Two-step go-link token resolver
    * Resolves `#t=v1...` go-links via API and sandboxed challenge webview for countdown and turnstile verification
* **Download Routing**: Smart download dispatch
    * In-app streaming with real-time byte counters and speed tracking for direct hosts
    * OS default-handler dispatch for cloud storage hosts
* **Host Blacklist**: Defunct and malicious host shielding
    * Rejects mirrors such as `gofile`, `zippyshare`, `cdnclick`, `anonfiles`, `uptobox`, `yourfilestore`, `qiwi`, `transfersh`
* **7-Zip Extraction**: High-performance multi-format archive extraction
    * Uses standalone 7-Zip console binaries (`7za`/`7z`/`7zz`) with real-time `-bsp1` progress parsing
* **Library Layout**: Organized flat library structure
    * `<library-root>/downloads/<archive>` staging and `<library-root>/installed/<slug>/app.json` manifests
* **Game Scanner & Launcher**: Automatic game discovery scanner with itch.io-compatible manifest generation
* **SQLite Queue**: Persistent sequential download queue
* **System Tray**: Minimize-to-tray background operation with quick-access context menu
* **Theming**: Runtime stylesheet switching across Nord, Dracula, and Material themes
* **Metadata Enrichment**: External metadata and artwork pipeline
    * Fetches posters and hero banners from SteamGridDB, VNDB, IGDB, itch.io, Steam, and IndieDB
    * Stores provider API keys securely in SQLite

### 🔄 Changed

### 🐛 Fixed

---

| Version | Title | Description |
| :---: | :---: | :---: |
| v0.4.1 | Resolver cleanup, unified extraction, installer titlebar | Removed browser resolver button; auto-extract all archive formats; matching installer titlebar |
| v0.4.0 | Accelerated downloads, installer UI fixes | Multi-stream downloads; expanded direct-stream hosts; fixed installer titlebar and console flashes |
| v0.3.3 | Resolver navigation fixes | Desktop user agent; removed premature queueing; in-window capture; loading state |
| v0.3.2 | Installer process isolation | Hidden main window during setup; stale-instance termination; deferred launch; clean WebView2 teardown |
| v0.3.1 | Installer naming and capture fixes | Platform-specific installer names; legacy NSIS cleanup; process termination; direct-archive capture |
| v0.3.0 | Unified installer wizard and UI refresh | Single-instance enforcement; background service; installer wizard; archive interception; 2-column layouts |
| v0.2.1 | CI packaging and SteamGridDB settings | Tauri v2 CI alignment; SteamGridDB API key persistence |
| v0.2.0 | Shortcuts, playtime tracking, and CI packaging | Native shortcuts; session tracking; library sorting; multi-platform CI packaging |
| v0.1.0 | Initial release | First public release with catalog, resolver, downloads, extraction, library, and theming |

---

**Last updated:** 2026-09-25
