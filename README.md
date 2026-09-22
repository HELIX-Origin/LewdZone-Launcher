<p align="center">

# LewdZone Launcher

</p>

A cross-platform launcher (Windows, Linux, macOS) for browsing, downloading,
and organizing games from [LewdZone](https://lewdzone.com).

It pairs a Tauri 2 desktop app (the primary product) with a **native Rust CLI**:
two entry points into the same Rust core (`src-tauri/src/`). Every GUI action
maps 1:1 to a headless, scriptable CLI subcommand backed by the same functions.
The tool never downloads files itself — it resolves real links and hands them
to an installed download manager (FDM, IDM, or a torrent client).

## Features

- Steam-like desktop app: **Store**, **Library**, **Downloads**, **Settings**
- Command-line engine with the full capability of the app, headless
- Scrapes games, tags, versions, and download links from LewdZone
- SQLite catalog storing go-link tokens (never resolved URLs)
- Pluggable download-manager adapters: FDM, IDM, uTorrent/BitTorrent
- Native shortcuts & icons per platform (`.lnk`, `.desktop`, `.app`) with
  SteamGridDB artwork
- Enriches thin pages from multiple content providers (VNDB, IGDB, Steam,
  itch.io, IndieDB): descriptions, screenshots, ratings, cover art

## Documentation

The full documentation suite lives in the [wiki](wiki/Home) folder:

- [Architecture](wiki/Architecture)
- [CLI Reference](wiki/CLI-Reference)
- [Download Managers](wiki/Download-Managers)
- [Content Providers](wiki/Content-Providers)
- [Development](wiki/Development)
- [Installing & Building](wiki/Installing-and-Building)
- [Agents ecosystem](wiki/Agents)