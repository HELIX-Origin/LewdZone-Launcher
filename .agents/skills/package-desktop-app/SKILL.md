---
name: package-desktop-app
description: Package the Tauri app into cross-platform installers (MSI/NSIS, .app/DMG, AppImage/deb/rpm) and prepare release artifacts. Use when cutting an official release or testing the distribution build.
---

# Package Desktop App

Build the Tauri app into native installers for distribution. The same
`src-tauri/` crate produces the binary; `tauri.conf.json` configures the bundle
formats per platform.

## Preconditions

- The app builds and passes all gates (`cargo test`, `npm run check`,
  `npm run test`).
- `tauri.conf.json` bundle configuration is correct (identifier, icon paths,
  category, short/long descriptions).
- Target OS build tools are installed:
  - **Windows:** WiX Toolset (MSI) and/or NSIS.
  - **macOS:** Xcode command-line tools for `.app` + DMG.
  - **Linux:** `dpkg-deb`/`rpm-build` for deb/rpm packages; AppImage tooling is
    bundled by Tauri.

## Steps

1. **Bump versions** (Rule 08): sync `package.json`, `package-lock.json`,
   `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.
2. **Update `CHANGELOG.md`** with the release section and summary table.
3. **Run the release build** on each target OS:
   ```bash
   npm run tauri build
   ```
4. **Collect artifacts** from `src-tauri/target/release/bundle/`:
   - Windows: `.msi`, `.nsis.exe`
   - macOS: `.app`, `.dmg`
   - Linux: `.AppImage`, `.deb`, `.rpm`
5. **Smoke-test the installer** on a clean VM or machine:
   - Install completes.
   - App launches to the Store view.
   - CLI binary answers `--version` and `--help`.
6. **Upload to GitHub Releases** with the version tag (e.g. `v0.1.0`) and
   attach all per-platform artifacts.

## Checkoff

- [ ] Version numbers synced across package.json, Cargo.toml, tauri.conf.json
- [ ] CHANGELOG.md updated for the target version
- [ ] `npm run tauri build` succeeds on each target OS
- [ ] Installer artifacts exist for Windows, macOS, and Linux
- [ ] Smoke tests pass on each platform
- [ ] GitHub Release created with artifacts attached
