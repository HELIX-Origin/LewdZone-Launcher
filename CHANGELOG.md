# 📜 LewdZone Launcher Changelog

Historical record of every change to the repository. Each release anchors to a
tag URL; commit entries link to their full commit. Newer releases are added at
the top; the current development state lives under `Unreleased`.

---

## ⏳ Unreleased

(No unreleased changes.)

---

## [v0.2.2](https://github.com/HELIX-Origin/LewdZone-Launcher/releases/tag/v0.2.2) — 2026-09-24

Patch release fixing game artwork display in the Library and Favorites views, implementing robust SQLite thumbnail caching fallbacks and re-enabling SteamGridDB provider artwork fetching.

### 🐛 Bug Fixes

- **Library Artwork Display:** Fixed missing cover artwork in the Library view by exposing `thumb_url` on `LibraryGame`, querying SQLite for cached thumbnails by `slug` or `post_id`, and rendering them properly in the Svelte view with `convertFileSrc` support.
- **Favorites Artwork Display:** Resolved artwork in Favorites by passing `thumb_url` into the `artwork_url` Tauri command and adding direct fallback to `game.thumb_url` in Svelte templates.
- **SQLite Artwork Fallback in Rust Core:** Updated `artwork_url` in the Tauri core to query SQLite `thumbnail_by_slug` and `thumbnail_by_post_id` when the requested `card.thumb_url` is absent.
- **SteamGridDB Provider Re-enabled:** Implemented the `Provider` trait for `SteamGridDb`, querying SteamGridDB's grids API using the SQLite-stored API key and caching downloaded grid artwork to disk and the `artwork_cache` table.

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