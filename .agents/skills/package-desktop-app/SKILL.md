---
name: package-desktop-app
description: Build the unified per-platform installer executable and prepare release artifacts. Use when cutting an official release or testing the distribution build.
---

# Package Desktop App

Build the Tauri app into the unified per-platform installer executable. The same
`src-tauri/` crate produces the binary; `scripts/build-installer.mjs` packages it
as `dist/installer/LewdZone-Setup-*` without using Tauri's native MSI/NSIS/.dmg
bundles.

## Preconditions

- The app builds and passes all gates (`cargo test`, `npm run check`,
  `npm run test`).
- `tauri.conf.json` is configured with the correct identifier and icon paths.
- Rust stable and Node.js LTS are installed.

## Steps

1. **Bump versions** (Rule 08): sync `package.json`, `package-lock.json`,
   `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.
2. **Update `CHANGELOG.md`** with the release section and summary table.
3. **Run the release build** on each target OS:
   ```bash
   npm run build:installer
   ```
4. **Collect artifacts** from `dist/installer/`:
   - Windows: `LewdZone-Setup-v<version>-windows-<arch>.exe`
   - macOS: `LewdZone-Setup-v<version>-macos-<arch>`
   - Linux: `LewdZone-Setup-v<version>-linux-<arch>`
   The script also emits generic names (`LewdZone-Setup.exe`,
   `LewdZone-Setup-windows-<arch>.exe`, etc.).
5. **Smoke-test the installer** on a clean VM or machine:
   - Install completes.
   - App launches to the Store view.
   - CLI binary answers `--version` and `--help`.
6. **Publish by pushing the annotated tag** (e.g. `v0.1.0`). The
   `.github/workflows/package.yml` CI workflow creates the GitHub Release and
   attaches the per-platform artifacts automatically.

## Checkoff

- [ ] Version numbers synced across package.json, package-lock.json, Cargo.toml, tauri.conf.json
- [ ] CHANGELOG.md updated for the target version
- [ ] `npm run build:installer` succeeds on each target OS
- [ ] Unified installer artifacts exist in `dist/installer/` for Windows, macOS, and Linux
- [ ] Smoke tests pass on each platform
- [ ] Annotated `v*` tag pushed and CI release workflow completed
