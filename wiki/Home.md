# 👋 Welcome to the LewdZone Launcher Wiki

LewdZone Launcher is a **cross-platform desktop launcher** (Windows, Linux,
macOS) for browsing, downloading, extracting, and organizing games from
[LewdZone](https://lewdzone.com). It is built around a **Tauri 2 app** — Rust
+ OS webview, Svelte frontend — whose binary also exposes a **native Rust CLI**:
one core, two entry points.

- A **Tauri 2 desktop app** — the primary product. Storefront-style views:
  **Store**, **Library**, **Downloads**, **Favorites**, and **Settings**, featuring system tray minimize, live extraction progress, and sandboxed link resolution.
- A **native Rust CLI** — the scriptable edition of the same engine.
  `lewdzone <command> --json` drives everything the app does.

The GUI and the CLI call the same Rust core functions; every GUI action maps
1:1 to a CLI command (Rule 03, Rule 13).

---

## 🛠️ Key Capabilities

- **Catalog Scraping:** Scrapes games, tags, versions, download links, and header details from LewdZone.
- **Go-Link Token Resolution:** Resolves ephemeral `#t=v1...` tokens through the site's two-step `start` → `reveal` API.
- **In-App Sandboxed Resolver:** Safely verifies protected download links inside an isolated, script-filtered webview window without third-party popups or adware.
- **Direct-File Streaming:** Streams direct file hosts (e.g. `fileknot`) inside the app with real-time byte counters and progress metrics.
- **OS Native Cloud Dispatch:** Hands cloud hosts (`mega`, `google`, `dropbox`, `mediafire`, `pixeldrain`) directly to the OS default handler or browser.
- **7-Zip Multi-Format Extraction:** Fast, multi-threaded decompression of `.zip`, `.7z`, `.rar`, `.tar`, and SFX archives using the standalone 7-Zip CLI with real-time progress tracking.
- **Flat Library Organization:** Downloads land directly under `<library-root>/downloads/` and installs unpack to `<library-root>/installed/<slug>/` with an `app.json` manifest.
- **System Tray Integration:** Runs silently in the background with tray icon controls (Open, Library, Downloads, Store, Settings, Check for Updates, Quit) and minimize-to-tray capability.
- **External Enrichment:** Augments thin LewdZone listings with external metadata and artwork from SteamGridDB, VNDB, IGDB, itch.io, Steam, and IndieDB.

---

## 🔗 Quick Links

| Topic | Where |
| --- | --- |
| First run & requirements | [Getting Started](Getting-Started) |
| Build & install instructions | [Installing & Building](Installing-and-Building) |
| System architecture | [Architecture](Architecture) |
| Command-line reference | [CLI Reference](CLI-Reference) |
| Downloads & streaming dispatch | [Downloads & Streaming](Download-Managers) |
| 7-Zip CLI setup & downloads | [Archive Extraction & 7-Zip](Archive-Extraction) |
| Info & art enrichment | [Content Providers](Content-Providers) |
| Config files & setting keys | [Configuration](Configuration) |
| Theme creation & customization | [Theme Development](Theme-Development) |
| Agent ecosystem & governance | [Agent Ecosystem](Agents) |
| Coding & diagram conventions | [Design Conventions](Design-Conventions) |
| Testing & QA workflow | [Testing & QA](Testing) |
| Security architecture | [Security](Security) |
| Release pipeline | [Release Process](Release-Process) |
| Common problems & solutions | [Troubleshooting](Troubleshooting) |

---

## 🗺️ Project Status

**Active Implementation & Verification.** Core scraping, resolver, download streaming, queue management with cancel/delete, 7-Zip extraction with real-time progress, system tray integration, and settings configuration are fully implemented, verified, and backed by a comprehensive Rust and frontend Vitest test suite.