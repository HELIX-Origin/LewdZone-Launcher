# 📜 LewdZone Launcher Changelog

Historical record of every change to the repository. Each release anchors to a
tag URL; commit entries link to their full commit. Newer releases are added at
the top; the current development state lives under `Unreleased`.

---

## ⏳ Unreleased

(No unreleased changes.)

---

## v0.1.0 — 2026-09-24

Initial release of LewdZone Launcher — a cross-platform desktop game launcher and native CLI engine for lewdzone.com.

### ✨ Added

- **feat(core): single Tauri 2 binary with native Rust CLI engine** (`lewdzone <cmd> --json`) sharing 1:1 parity with the desktop GUI (Rule 03, Rule 13).
- **feat(scraper): rate-limited LewdZone web scraper** for catalog pagination, game cards, genre clouds, version tabs, and download sources.
- **feat(resolver): two-step go-link token resolver** via `start` → `reveal` API with dead and malicious host blacklist protection (`gofile`, `gofiles`, `zippyshare`, `cdnclick`, `anonfile`, `anonfiles`, `anonzip`, `uptobox`, `yourfilestore`, `qiwi`, `transfersh`).
- **feat(resolver): in-app sandboxed webview resolver** for countdown and captcha challenges, blocking popups, adware, and tracking scripts.
- **feat(download): in-app direct file streaming** for `fileknot` with real-time byte counters and download speed calculation.
- **feat(download): OS default handler dispatch** for cloud storage hosts (`mega`, `google`, `dropbox`, `mediafire`, `pixeldrain`, `workupload`, `uploadhaven`, `transfaze`).
- **feat(extract): multi-format archive extraction** using the standalone 7-Zip console executable (`7za`/`7z`/`7zz`) with real-time percentage progress parsing (`-bsp1`), silent background execution, and cancellation.
- **feat(folder): flat library directory layout** organizing downloads into `<library-root>/downloads/<archive>` and installs into `<library-root>/installed/<slug>/` with an `app.json` manifest.
- **feat(library): automatic game discovery & launch** via "Scan Games" (`games-dir`), itch.io-style manifest generation, and game process launching.
- **feat(queue): persistent SQLite download queue** supporting sequential workers, pause/resume, and active job cancel/delete controls.
- **feat(tray): system tray integration** featuring custom tray icon, context menu (Open, Library, Downloads, Store, Settings, Quit), and minimize-to-tray window management.
- **feat(theme): dynamic runtime theme engine** shipping Nord, Dracula, and Material reference skins with dynamic CSS token switching without app restart.
- **feat(content): external metadata & artwork enrichment** pipeline supporting SteamGridDB, VNDB, IGDB, itch.io, Steam, and IndieDB with secure SQLite secret storage.
- **docs(wiki): comprehensive GitHub wiki suite** including getting started, 7-Zip CLI setup guide, architecture, CLI reference, configuration, theme development, troubleshooting, and testing.

### 🛠 Fixed

- **fix(gui): remove redundant profile icon** from top navigation bar.
- **fix(folder): flatten library downloads and installed folders**, removing cluttered engine subdirectories while retaining backward-compatible legacy detection.
- **fix(resolver): replace restrictive host allowlist** with an open blacklist targeting defunct and malicious mirrors.
- **fix(testing): hermetic offline test suites** passing 183 Rust tests and 31 Vitest frontend tests.

### 🚫 Removed

- Unreliable external download-manager adapters (FDM, IDM, torrent) in favor of integrated streaming and native OS dispatch.
- Android platform downloads in the desktop application.