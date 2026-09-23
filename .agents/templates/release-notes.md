---
name: release-notes
description: Template for GitHub release notes
---

# Release Notes: v{{ version }}

**Release date:** {{ YYYY-MM-DD }}

## Highlights

{{ 2–3 sentence summary. }}

## What's new

- {{ Feature / change }}
- {{ Feature / change }}

## Fixed

- {{ Bug fix }}

## Removed / deprecated

- {{ Removed feature }}

## Install

Download the installer for your platform from the Assets section below:

- Windows: `LewdZone-Launcher_{{ version }}_x64-setup.exe` / `.msi`
- macOS: `LewdZone-Launcher_{{ version }}_x64.dmg`
- Linux: `LewdZone-Launcher_{{ version }}_amd64.AppImage` / `.deb`

## Verification

- `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` — green
- `npm run check`, `npm run test` — green
- `npm run tauri build` — green

## Full changelog

See [CHANGELOG.md](../CHANGELOG.md).
