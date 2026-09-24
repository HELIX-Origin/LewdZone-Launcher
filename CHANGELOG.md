# 📜 LewdZone Launcher Changelog

Historical record of every change to the repository. Each release anchors to a
tag URL; commit entries link to their full commit. Newer releases are added at
the top; the current development state lives under `Unreleased`.

---

## ⏳ Unreleased

(No unreleased changes.)

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