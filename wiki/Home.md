# Welcome to the LewdZone-Launcher Wiki

> See the [_Sidebar](_Sidebar) for navigation. This wiki is synced with the
> GitHub Wiki — all internal links are relative and omit the `.md` extension.

LewdZone-Launcher is a **cross-platform desktop launcher** (Windows, Linux,
macOS) for browsing, downloading, and organizing games from
[LewdZone](https://lewdzone.com). It is built around a **Tauri 2 app** — Rust
+ OS webview, Svelte frontend — whose binary also exposes a **native Rust CLI**:
one core, two entry points.

- A **Tauri 2 desktop app** — the primary product. Steam-like pages: **Store**,
  **Library**, **Downloads**, **Settings**.
- A **native Rust CLI** — the scriptable edition of the same engine.
  `lewdzone-launcher <command> --json` drives everything the app does.

The GUI and the CLI call the same Rust core functions; every GUI action maps
1:1 to a CLI command.

## What it does

- **Scrapes** games, tags, versions, download tables, and metadata from
  LewdZone.
- **Resolves** go-link tokens to real file URLs (the site blocks naive
  follow-through; the tool uses the site's own two-step `start`→`reveal` API).
- **Dispatches downloads to a real manager**: FDM, IDM, or a torrent client
  (uTorrent/BitTorrent). The tool never downloads files itself.
- **Organizes** downloaded files into `<DownloadRoot>/Games/<Title>/` with
  canonical names.
- **Builds native shortcuts + icons** per OS (`.lnk`, `.desktop`, `.app`) using
  SteamGridDB artwork.
- **Enriches** thin LewdZone pages from external content providers (VNDB, IGDB,
  Steam Storefront, itch.io, IndieDB): descriptions, screenshots, ratings, art.

## Quick links

| Topic | Where |
| --- | --- |
| First run & requirements | [Getting Started](Getting-Started) |
| Build & install | [Installing & Building](Installing-and-Building) |
| How the pieces fit together | [Architecture](Architecture) |
| Command-line reference | [CLI Reference](CLI-Reference) |
| FDM / IDM / torrent handling | [Download Managers](Download-Managers) |
| Info + art enrichment | [Content Providers](Content-Providers) |
| Config files & options | [Configuration](Configuration) |
| Agent ecosystem (governance) | [Agent Ecosystem](Agents) |
| Coding & diagram conventions | [Design Conventions](Design-Conventions) |
| Testing workflow | [Testing & QA](Testing) |
| Security assumptions | [Security](Security) |
| Releases & versioning | [Release Process](Release-Process) |
| Common problems | [Troubleshooting](Troubleshooting) |

## Project status

Design/documentation phase. The living spec is the `.agents` ecosystem in the
repository root; the wiki is the human-facing distillation of it.