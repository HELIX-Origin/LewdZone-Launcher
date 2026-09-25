# LewdZone Launcher v{{ version }}

**Release date:** {{ YYYY-MM-DD }}

## ✨ Highlights

{{ 2–3 sentence summary for users. }}

## 🚀 Key Improvements & Features

- **{{ Feature name }}** — {{ Feature description }}
- **{{ Feature name }}** — {{ Feature description }}

## ✅ Changed

- {{ Change description }}

## 🐛 Fixed

- {{ Bug fix description }}

## 📦 Install & Upgrading

Download the installer for your platform from the Assets section below:

- Windows: `LewdZone-Setup-v{{ version }}-windows-x64.exe`
- macOS: `LewdZone-Setup-v{{ version }}-macos-arm64` (or `macos-x64` if available)
- Linux: `LewdZone-Setup-v{{ version }}-linux-x64`

Upgrade in place: run the new installer over your existing installation. The unified installer will repair/replace files and update shortcuts.

## Verification

- `cargo check` + `cargo test` (src-tauri) — passed
- `npm run check` (svelte-check) — 0 errors, 0 warnings
- `npm run test` (vitest) — passed
- `npm run build` — adapter-static site generated successfully

## 📄 Changes & Commits

- `{{ commit title / description }}`
- `{{ commit title / description }}`

Full commit history: `git log --oneline {{ prev_tag }}..v{{ version }}`

