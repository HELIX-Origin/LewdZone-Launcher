---
name: gui-build-loop
description: Run the local Tauri development and build loop (npm install, npm run tauri dev, npm run tauri build) and diagnose common frontend/Rust build failures. Use when iterating the GUI or verifying that the bundle compiles.
---

# GUI Build Loop

Run the standard Tauri frontend + Rust build loop locally. The GUI is a SvelteKit
app in `src/` backed by the Rust core in `src-tauri/`. Both sides must compile
and the bundle must produce a working binary.

## Preconditions

- Node.js and npm are installed.
- Rust toolchain is installed (Rule 01).
- `src-tauri/` dependencies are resolvable (`cargo` can fetch crates).

## Steps

1. **Install frontend dependencies** (repo root):
   ```bash
   npm install
   ```
2. **Run the dev loop** (compiles Rust in debug, serves Svelte, opens app window):
   ```bash
   npm run tauri dev
   ```
   - The first run builds the Rust crate; subsequent runs are incremental.
   - Watch the terminal for both Vite and cargo errors.
3. **Build a release bundle** (produces installer artifacts):
   ```bash
   npm run tauri build
   ```
   - Artifacts land in `src-tauri/target/release/bundle/`.
4. **Diagnose common failures**:
   - `svelte-check` errors → run `npm run check` and fix TypeScript/Svelte issues.
   - `cargo clippy` warnings → run `cargo clippy -- -D warnings` in `src-tauri/`.
   - Missing system dependencies on Linux → install `libwebkit2gtk-4.1-dev`,
     `libappindicator3-dev`, etc. (see Tauri Linux prerequisites).
   - Windows NSIS/MSI missing → install WiX Toolset / NSIS and ensure they are on
     `PATH`.

## Checkoff

- [ ] `npm install` completes without lockfile conflicts
- [ ] `npm run tauri dev` launches the app window
- [ ] `npm run check` reports 0 errors / 0 warnings
- [ ] `npm run test` passes (Vitest)
- [ ] `npm run tauri build` produces a bundle for the host OS
- [ ] `cargo fmt --check` and `cargo clippy -- -D warnings` are clean
