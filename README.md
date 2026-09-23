<p align="center">

# 🎮 LewdZone Launcher

  <img src="./.github/assets/lewdzone-launcher-banner.jpg" alt="LewdZone Launcher" width="100%">
</p>

> 🚀 **Cross-platform game launcher** for [LewdZone](https://lewdzone.com) — browse, download, and organize
> without ever downloading files yourself.

A **Tauri 2 desktop app** (Windows, Linux, macOS) paired with a **native Rust CLI** — two entry points into the
same Rust core (`src-tauri/src/`). Every GUI action maps 1:1 to a headless, scriptable CLI subcommand. The tool
resolves real links and hands them to an installed download manager (FDM, IDM, or a torrent client).

---

## ✨ Features

| | |
|---|---|
| 🖥️ **Storefront-style desktop app** | Store, Library, Downloads, Settings views |
| ⌨️ **Native CLI engine** | Full app capability, headless & scriptable, `--json` output |
| 🕷️ **LewdZone scraper** | Games, tags, versions, and download links |
| 🗄️ **SQLite catalog** | Stores go-link tokens — never resolved URLs |
| 🔌 **DM adapters** | FDM, IDM, uTorrent/BitTorrent, auto-detection |
| 🖼️ **Native shortcuts & icons** | `.lnk`, `.desktop`, `.app` + SteamGridDB artwork |
| 📚 **Content enrichment** | VNDB, IGDB, Steam, itch.io, IndieDB metadata & art |

---

## 🚀 Quick Start

### Build & run the GUI

```bash
npm install            # frontend deps (repo root)
npm run tauri dev      # launch the desktop app
```

### Use the CLI

```bash
cargo build            # from src-tauri/ — builds target/debug/lewdzone

lewdzone sync          # refresh the catalog (incremental)
lewdzone sync --full   # full resync of every page
lewdzone list --json   # catalog from the SQLite DB
lewdzone info --game treasure-of-nadia
lewdzone dm fdm        # select the active download manager
lewdzone download --game treasure-of-nadia --json
```

> 💡 Every GUI action maps to a CLI subcommand — learn one, you know the other (Rule 13).

---

## 📚 Documentation

The full documentation suite lives in the [wiki](wiki/Home) folder:

- [Architecture](wiki/Architecture)
- [CLI Reference](wiki/CLI-Reference)
- [Download Managers](wiki/Download-Managers)
- [Content Providers](wiki/Content-Providers)
- [Development](wiki/Development)
- [Installing & Building](wiki/Installing-and-Building)
- [Agents ecosystem](wiki/Agents)

---

## ⚖️ Legal

View the [License](LICENSE.md), [Privacy Policy](PRIVACY.md), [Terms of Service](TOS.md), and
[Security Policy](SECURITY.md).