# LewdZone-Launcher

A cross-platform launcher (Windows, Linux, macOS) for browsing, downloading,
and organizing games from [LewdZone](https://lewdzone.com).

It pairs a Tauri 2 desktop app (the primary product) with a modular Python CLI
engine. The app spawns the CLI as a sidecar process and drives it over a
JSON/JSONL protocol, so every GUI action maps 1:1 to a headless, scriptable
command. The tool never downloads files itself — it resolves real links and
hands them to an installed download manager (FDM, IDM, or a torrent client).

## Features

- Steam-like desktop app: **Store**, **Library**, **Downloads**, **Settings**
- Command-line engine with the full capability of the app, headless
- Scrapes games, tags, versions, and download links from LewdZone
- SQLite catalog storing go-link tokens (never resolved URLs)
- Pluggable download-manager adapters: FDM, IDM, uTorrent/BitTorrent
- Native shortcuts & icons per platform (`.lnk`, `.desktop`, `.app`) with
  SteamGridDB artwork

## Documentation

The full documentation suite lives in the [wiki](wiki/Home) folder:

- [Architecture](wiki/Architecture)
- [CLI Reference](wiki/CLI-Reference)
- [Download Managers](wiki/Download-Managers)
- [Development](wiki/Development)
- [Installing & Building](wiki/Installing-and-Building)
- [Agents ecosystem](wiki/Agents)