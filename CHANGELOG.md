# 📜 LewdZone Launcher Changelog

Historical record of every change to the repository. Each release anchors to a
tag URL; commit entries link to their full commit. Newer releases are added at
the top; the current development state lives under `Unreleased`.

---

## ⏳ Unreleased

(No unreleased changes.)

---

## [v0.3.1](https://github.com/HELIX-Origin/LewdZone-Launcher/releases/tag/v0.3.1) — 2026-09-24

Patch release fixing installer file naming collisions across platforms, adding automatic stale instance termination during installation/uninstallation, migrating legacy install directories, and improving direct archive link interception.

### 🐛 Bug Fixes & Improvements

- **Platform-Specific Installer Naming:** Updated the installer packager (`scripts/build-installer.mjs`) to generate clearly distinguished binaries for each operating system and architecture (`LewdZone-Setup-v0.3.1-windows-x64.exe`, `LewdZone-Setup-v0.3.1-linux-x64`, `LewdZone-Setup-v0.3.1-macos-x64`/`arm64`). Prevents CI asset collisions between Linux and macOS and makes platform targets immediately clear to users.
- **Running Instance Termination During Install/Uninstall:** Enhanced the installer engine (`core/installer.rs`) to automatically detect and terminate any running `lewdzone` processes before deploying files or uninstalling. Resolves file locks (`Access is denied`) and prevents old background/tray instances from intercepting single-instance focus on update.
- **Legacy Installation Migration:** Added detection and automatic cleanup for legacy Tauri NSIS directories (`%LOCALAPPDATA%\LewdZone Launcher\`) to prevent stale executable versions from persisting alongside new unified installs.
- **Instant In-App Capture for Direct Archives:** Enhanced "Open in App" in the Secure Resolver child window to detect direct archive URLs (`.zip`, `.7z`, `.rar`, `.001`, `.part1.rar`, etc.) and immediately enqueue them to the download worker, seamlessly closing the resolver window. Expanded archive format detection in Tauri's webview download interceptor.

---

## [v0.3.0](https://github.com/HELIX-Origin/LewdZone-Launcher/releases/tag/v0.3.0) — 2026-09-24

Milestone release featuring the modern in-app Unified Installer / Uninstaller wizard, child-window archive download interception, 2-column store and library layouts with dedicated metadata sidebars, clean synopsis scraping, and clean game title handling across manifests and UI.

### ✨ Highlights

- **Single-Instance Enforcement (`tauri-plugin-single-instance`):** Integrated single-instance plugin ensuring launching duplicate instances automatically focuses the existing running application window.
- **Client Cache & Smart Polling (`src/lib/stores/clientCache.ts`):** Centralized Svelte stores for catalog, details, library, favorites, and settings with smart conditional polling that eliminates redundant page reloads and network queries while keeping download progress responsive.
- **Canonical Release Descriptors & Title Case Formatting:** Added dedicated `title` column to `queue_job` (DB migration `008_queue_job_title`), ensuring Title Case game names across the Downloads page and resolving chapter/version separation (`Version 1.01 Chapter 1-4`) across the Secure Ad-Free Resolver child window, download queue, and filesystem archives.
- **Persistent Background Service (`src-tauri/src/core/service.rs`):** A dedicated background worker running alongside the launcher application, coordinating task queue execution, external host URL resolution (Pixeldrain, Mediafire, Fileknot), streaming downloads, and automatic ingestion of completed archives from the downloads directory.
- **Privacy & Secure Database Architecture:** Ensured the SQLite database (`lewdzone.db`) is strictly local and never bundled or committed to the repository. The installer checks and initializes a fresh database schema on install (preserved if it already exists), keeping user tokens, secrets, and library progress completely private in `%APPDATA%\lewdzone\`.
- **Resolver Window & In-App Interception:** Fixed redirect loop in the go-token resolver window. Added an "Open in App" option alongside "Open in Browser" so users can navigate hosting landing pages directly in-app, where the Tauri webview's `on_download` hook captures the archive trigger automatically.
- **Modern Unified Installer Wizard (`LewdZone-Setup.exe`):** Replaced legacy WiX/NSIS installers with an in-app setup and maintenance wizard (`src/routes/installer/+page.svelte`). Handles clean installs, component selection, start menu shortcuts, uninstallation, and maintenance mode directly in a single lightweight binary.
- **Child-Window Archive Interception:** Intercepts archive download requests (`.zip`, `.7z`, `.rar`, `.exe` SFX, `.tar.gz`) clicked within the redirect resolver window via Tauri 2's `on_download` hook. Cancels the OS browser dialog, captures the direct URL, and streams the archive directly in-app to `<LibraryRoot>/downloads/` with live byte progress, automatically queuing extraction upon completion.
- **2-Column Layout & Metadata Sidebars:** Restructured both Store (`/store/[slug]`) and Library (`/library/[slug]`) views into a modern 2-column layout (`1fr 320px`) with a sticky metadata card. Displays game status badges, developer, engine, size on disk, rating, censorship, platform badges, and direct external links to LewdZone, VNDB, Steam, and itch.io.
- **Clean Story Synopsis:** Upgraded the scraper to parse clean game plot and synopsis paragraphs directly from `.content-block.main-content`, eliminating SEO promotional boilerplate (such as version numbers, file sizes, walkthrough adverts, and download links).
- **Clean Library Titles:** Sanitized game titles across local folder scanning, manifest generation (`app.json`), and UI display, stripping archive extensions, version suffixes, and platform tags.

### 🚀 Key Improvements & Features

- **Queue & Download Engine:** Added `enqueue_intercept()` and `"intercept"` processing branch in `core/queue.rs`, enabling direct-URL streaming and extraction dispatch without going through go-link resolvers.
- **Library & Title Parsing:** Added `clean_archive_stem()`, `clean_folder_title()`, `format_display_title()`, and frontend format module (`src/lib/format.ts`) to ensure consistent display titles across all views.
- **LewdClips Removal:** Completely removed all third-party video and external clip links and fixture references from the codebase.
- **CI / Distribution Build:** Added `scripts/build-installer.mjs` and updated GitHub Actions packaging to build and publish the unified setup binary to `dist/installer/`.

---

## [v0.2.1](https://github.com/HELIX-Origin/LewdZone-Launcher/releases/tag/v0.2.1) — 2026-09-24

Patch release fixing cross-platform CI packaging builds for Linux and macOS, aligning workflow configuration with the official Tauri v2 GitHub Actions guide, and adding SteamGridDB API key settings persistence to the SQLite secret table.

### 🐛 Bug Fixes

- **CI Packaging Pipeline:** Fixed `ENOENT` spawn failure on Linux and macOS during Tauri release bundling. `beforeBuildCommand` in `tauri.conf.json` was switched to standard `npm run build` and `scripts/build-with-log.mjs` was updated to enforce `shell: true` across all operating systems.
- **Official Tauri v2 Alignment:** Updated `.github/workflows/package.yml` per the official Tauri v2 pipeline guide, installing dual macOS targets (`aarch64-apple-darwin` and `x86_64-apple-darwin`), setting `node-version: lts/*`, and providing native system libraries (`libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf`).
- **Tauri Native Artifact Dropping:** Replaced the legacy `build/` root directory and `copy-installers.mjs` script with native bundle collection directly from `src-tauri/target/release/bundle/`. Cleaned `.gitignore` and `package.json` scripts.
- **Workflow Dispatch Safety:** Scoped `tagName` in `tauri-action` to `v*` tag push events so manual packaging test dispatches upload artifacts cleanly without attempting to publish duplicate release tags.

### ✨ Features

- **SteamGridDB API Key Settings:** Added a new *Content Providers* section to the Settings page. Users can input their SteamGridDB API key with masked password display. Keys are securely stored in the SQLite `secret` table via `settings_set` (`secret: true`) and never written to `config.json`. Displays live *● Key saved* / *○ Not configured* status indicators and provides Save and Clear controls.

---

## [v0.2.0](https://github.com/HELIX-Origin/LewdZone-Launcher/releases/tag/v0.2.0) — 2026-09-24

Milestone v0.2.0 release introducing native operating system desktop shortcuts, gameplay playtime and session tracking, library sort controls, and automated cross-platform CI/CD packaging.

### ✨ Highlights

- **Native Per-OS Shortcuts:** Generate desktop and start menu shortcuts with a single click in the Library or via `lewdzone shortcuts <slug>` from the terminal. On Windows, shortcuts point directly to the game binary with the executable's embedded icon index.
- **Playtime & Session Tracking:** The launcher monitors game child processes in detached background worker threads, calculating elapsed playtime upon exit and persisting metrics to SQLite (`game_stats`).
- **Library Sorting:** Sort games by A–Z, Recently Played, Most Played, or Recently Installed, accompanied by formatted playtime badges and last-played timestamps.
- **Automated Multi-Platform Packaging:** Multi-OS CI/CD packaging workflow (`.github/workflows/package.yml`) building Windows (NSIS + MSI), macOS (.app + DMG), and Linux (AppImage + DEB) packages.

### 🚀 Key Improvements & Features

- **Database:** Added migration `006_game_playtime` creating `game_stats` table tracking `playtime_seconds`, `play_count`, and `last_played_at`.
- **Core Engine:** Updated `launch.rs` with child process wait listener, and updated `library.rs` (`InstalledApp`) with playtime statistics loading.
- **Shortcuts Generator:** Added `shortcuts.rs` implementing cross-platform shortcut generation for Windows (`.lnk`), Linux (`.desktop`), and macOS (`.command`).
- **CLI Commands:** Added `lewdzone shortcuts [SLUG]` command and updated CLI parser with full feature parity.
- **UI / Svelte Frontend:** Added shortcut button on game cards, shortcut toast alert, sort dropdown, and playtime badges.
- **Documentation:** Added `Shortcuts & Playtime Tracking` guide to the GitHub Wiki and updated CLI reference.

---

## [v0.1.0](https://github.com/HELIX-Origin/LewdZone-Launcher/releases/tag/v0.1.0) — 2026-09-24

Initial release of LewdZone Launcher — a cross-platform desktop game launcher and native CLI engine for lewdzone.com.

### ✨ Features & Capabilities

- **Unified Desktop GUI & Native Rust CLI**: Built on Tauri 2 and Rust, sharing a single core engine between the desktop GUI and the standalone command-line client (`lewdzone <cmd> --json`) with 100% feature parity.
- **Built-in Catalog Browsing & Search**: Built-in scraper for lewdzone.com catalog with tag/genre filtering, engine selectors (Ren'Py, RPG Maker, Unity, HTML), release status, and pagination.
- **Two-Step Go-Link Token Resolver**: Automatic resolution of `#t=v1...` go-links via API and sandboxed challenge webview for countdown and turnstile verification.
- **Smart Download Routing**: In-app streaming with real-time byte counters and speed tracking for direct hosts (`fileknot`), plus seamless OS default handler dispatch for cloud storage hosts (`mega`, `google`, `dropbox`, `mediafire`, `pixeldrain`, `workupload`, `uploadhaven`, `transfaze`).
- **Defunct & Malicious Host Shielding**: Host blacklist rejecting dead, defunct, or malicious mirrors (`gofile`, `zippyshare`, `cdnclick`, `anonfiles`, `uptobox`, `yourfilestore`, `qiwi`, `transfersh`).
- **High-Performance 7-Zip CLI Extraction**: High-speed, multi-format archive extraction (`.7z`, `.zip`, `.rar`, `.tar`, `.tar.xz`) utilizing standalone 7-Zip console binaries (`7za`/`7z`/`7zz`) with real-time progress parsing (`-bsp1`) and background execution.
- **Organized Flat Library Layout**: Clean file hierarchy (`<library-root>/downloads/<archive>` and `<library-root>/installed/<slug>/app.json`) removing engine subfolder clutter.
- **Game Scanner & Launcher**: Automatic game discovery scanner, itch.io-compatible manifest generation, and game launch process tracking.
- **Persistent SQLite Queue**: Sequential download queue with pause, resume, and cancellation controls.
- **System Tray Integration**: Minimize-to-tray background operation with quick access context menu (Open, Library, Downloads, Store, Settings, Quit).
- **Dynamic Theming**: Runtime stylesheet switching across Nord, Dracula, and Material themes without restarting the app.
- **External Metadata & Artwork Enrichment**: Metadata and artwork pipeline fetching posters and hero banners from SteamGridDB, VNDB, IGDB, itch.io, Steam, and IndieDB with secure SQLite secret storage.
- **Offline Test Suite & Verification**: 100% passing hermetic test suites with 183 Rust core tests and 31 Svelte Vitest frontend tests.