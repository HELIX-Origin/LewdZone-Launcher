<div align="center">

# 🎮 LewdZone Launcher

  <img src="./.github/assets/lewdzone-launcher-banner.jpg" alt="LewdZone Launcher" width="100%">
</div>

> 🚀 **Cross-platform game launcher** for [LewdZone](https://lewdzone.com) — browse, download, and organize
> games with zero download-manager setup.

A **Tauri 2 desktop app** (Windows, Linux, macOS) paired with a **native Rust CLI** — two entry points into the
same Rust core (`src-tauri/src/`). Every GUI action maps 1:1 to a headless, scriptable CLI subcommand. The tool
resolves go-link tokens into real file URLs, streams direct-file hosts in-app with live progress, and hands every
other resolved URL to the OS default handler (installed cloud app or browser).

---

## ✨ Features

| | |
|---|---|
| 🖥️ **Storefront-style desktop app** | Store, Library, Downloads, Settings views |
| ⌨️ **Native CLI engine** | Full app capability, headless & scriptable, `--json` output |
| 🕷️ **LewdZone scraper** | Games, tags, versions, and download links |
| 🗄️ **SQLite catalog** | Stores go-link tokens — never resolved URLs |
| 📥 **In-app streaming + native dispatch** | Direct-file hosts stream with byte progress; cloud/page hosts open in the OS default handler |
| 🖼️ **Native shortcuts & icons** | `.lnk`, `.desktop`, `.app` + SteamGridDB artwork |
| 📚 **Content enrichment** | VNDB, IGDB, Steam, itch.io, IndieDB metadata & art |

---

## 🚀 Quick Start

### Build & run the GUI

```bash
npm install                                           # frontend deps (repo root)
npm run tauri dev                                     # launch the desktop app
```

### Use the CLI

```bash
cargo build                                           # from src-tauri/ — builds target/debug/lewdzone

lewdzone sync                                         # refresh the catalog (incremental)
lewdzone sync --full                                  # full resync of every page
lewdzone list --json                                  # catalog from the SQLite DB
lewdzone info --game treasure-of-nadia
lewdzone download --game treasure-of-nadia --json     # direct hosts stream in-app, others open natively
```

> 💡 Every GUI action maps to a CLI subcommand — learn one, you know the other (Rule 13).

---

## 📚 Documentation

The full documentation suite lives in the [wiki](../../wiki/Home) folder:

- [Architecture](../../wiki/Architecture)
- [CLI Reference](../../wiki/CLI-Reference)
- [Downloads & In-App Streaming](../../wiki/Download-Managers)
- [Content Providers](../../wiki/Content-Providers)
- [Development](../../wiki/Development)
- [Installing & Building](../../wiki/Installing-and-Building)
- [Agents ecosystem](../../wiki/Agents)

---

## ⚖️ Legal

View the [License](LICENSE.md), [Privacy Policy](PRIVACY.md), [Terms of Service](TOS.md), and
[Security Policy](SECURITY.md).