# 📊 Task Progress Log

**Task**: Engine-Organized Game Library Structure, System Tray with Context Menu, Multi-Format Archive Auto-Extraction, & Work Queue Cleanup  
**Updated**: 2026-09-23T18:40:00-07:00  
**Status**: COMPLETED

---

## 🎯 Objectives & User Requirements

1. **System Tray Icon with Context Menu**:
   - Built-in system tray icon using `TrayIconBuilder` with proper context menu:
     - `Show LewdZone` ("show"): Unminimizes, shows, and focuses main window.
     - `Minimize to Tray` ("minimize"): Hides the main window to tray.
     - `Quit` ("quit"): Calls `app.exit(0)` to cleanly terminate the application.
   - Left-click on the tray icon toggles window visibility (shows if hidden, hides if visible).
   - **Full-Bleed Tray Icon (Properly Sized)**: Tightly cropped the 50% empty transparent margins from the app icon down to its actual content bounds, generating dedicated high-res assets (`tray-icon.png`, `tray-icon-64.png`, `tray-icon-32.png`) so the icon graphic renders at 100% full scale in the taskbar tray instead of being shrunk by empty margins.
   - Window Close Interception: Closing the main window (`WindowEvent::CloseRequested`) hides the window (`api.prevent_close()`, `window.hide()`) rather than killing in-flight downloads/extractions.
   - Application Menu (`File` menu):
     - Added `Minimize to Tray` option (`getCurrentWindow().hide()`).
     - Wired `Quit` option to invoke `app_quit` (`app.exit(0)`) so the app can be exited cleanly.

2. **Engine-Organized Game Library Structure (`G:\LewdZone`)**:
    - **`downloads/` folder**: Stores downloaded zip and multi-format archive files organized directly by engine: `downloads\<engine>\<archive>` (e.g. `downloads/renpy/<archive>.zip`). Because archives are individual self-contained files with distinct filenames, a slug subfolder is not needed—organizing by engine alone keeps downloads clean and flat.
    - **`installed/` folder**: Extracted game folders organized with game slug subfolders by engine: `installed\<engine>\<slug>\` (e.g. `installed/renpy/<slug>/`). The slug folder is essential here to cleanly isolate the game's extracted files, assets, executables, and `app.json` manifest so games never collide or overwrite each other.
    - **`app.json` in game slug folder**: Manifest is saved right inside the game slug folder alongside the extracted game files/folders (e.g. `installed/renpy/<slug>/app.json`).
    - **Engine Path Detection**: Automatically detects the engine from the directory structure (e.g. `downloads/<engine>/...` or `installed/<engine>/...`) or the database engine tag (`Ren'Py` -> `renpy`, `RPG Maker` -> `rpgm`, `Unity` -> `unity`, `Unreal Engine` -> `unreal`, `HTML/NW.js` -> `html`, `Godot` -> `godot`, `Flash` -> `flash`, `WOLF RPG` -> `wolfrpg`, etc.).

3. **Automatic Directory & Engine Subfolder Creation**:
   - Whenever the game library folder is set (via `library-root` or `games-dir` in Settings or during `scan_games_dir`), the `downloads/` and `installed/` folders and all standard engine subfolders (`renpy`, `rpgm`, `unity`, `unreal`, `html`, `godot`, `flash`, `wolfrpg`, `other`) are automatically created on disk ready for user use.

4. **Multi-Format Archive Auto-Extraction & Non-Overlapping Layout**:
   - Supports multi-format archives: `.zip`, `.tar`, `.7z`, `.tar.gz`, `.tgz`, `.tar.bz2`, `.tbz2`, `.tar.xz`, `.txz`, `.rar`, and self-extracting (`.exe` SFX) archives.
   - Dual-engine extraction: fast native in-app zip extraction (`core::extract::extract_zip_with_progress`) with exact byte-by-byte progress callback, plus system tool fallback (`core::extract::extract_via_system_tool`) using `tar`/`bsdtar`, `7z`, or SFX runner.
   - **HTML Landing Page Detection**: Identifies HTML pages disguised as `.zip` (e.g. FileKnot landing pages), preventing extraction failures and surfacing clear user-facing errors.
   - Extracted games are organized into `installed/<engine>/<slug>/` using the canonical game slug, guaranteeing games never overlap each other.
   - User archives are left untouched.

5. **Convert Download Queue into a Unified Progress Queue ("Queue")**:
   - Unified `/downloads` into a Progress Queue ("Queue" in navigation) that manages both active file downloads and in-progress archive extractions.
   - Added `Extracting` and `Completed` statuses with real-time byte counters, percentage bars, elapsed time, and status badges.
   - Added "Clear Finished" action to purge completed and failed jobs.

6. **Clear Completed Items in Tracking Files**:
   - Cleared all completed checklist items from `TODO.md` to keep the active work queue focused.
   - Cleared resolved bug archives from `BUGS.md`.
   - Cleared completed milestones from `ROADMAP.md` Done section.

7. **Remove Unused Profile Icon**:
   - Removed the profile avatar button from the custom title bar in `src/routes/+layout.svelte` and cleaned up its CSS rules and DOM ordering.
   - Updated frontend unit tests in `src/lib/__tests__/page.test.ts` to assert the profile icon is removed.

---

## 📋 Step-by-Step Implementation & Verification Checklist

- [x] **System Tray Icon & Window Lifecycle (`src-tauri/src/lib.rs` & `src/routes/+layout.svelte`)**:
  - [x] Enabled `tray-icon` cargo feature in `tauri` dependency.
  - [x] Built native system tray with context menu items: `Show LewdZone`, `Minimize to Tray`, Separator, `Quit`.
  - [x] Added left-click tray icon toggle (toggle show/hide).
  - [x] Intercepted `WindowEvent::CloseRequested` to hide the window instead of destroying in-flight tasks.
  - [x] Created `app_quit` command to cleanly exit when requested.
  - [x] Updated UI `File` menu with `Minimize to Tray` and `Quit`.
- [x] **Engine Folder Conventions & Auto-Creation (`core::folder`)**:
  - [x] Defined `STANDARD_ENGINES` (`renpy`, `rpgm`, `unity`, `unreal`, `html`, `godot`, `flash`, `wolfrpg`, `qsp`, `twine`, `tyrano`, `rags`, `tads`, `java`, `python`, `webgl`, `other`).
  - [x] Implemented `engine_to_folder` to normalize engine strings to folder slugs.
  - [x] Implemented `engine_download_target` for direct `downloads\<engine>\<archive>` organization.
  - [x] Implemented `engine_install_dir` for `installed\<engine>\<slug>\` organization.
  - [x] Implemented `initialize_library_structure` to create `downloads/<engine>` and `installed/<engine>` on disk.
  - [x] Implemented `installed_root` and updated `download_root` to resolve effective roots.
- [x] **Settings Auto-Initialization (`core::settings`)**:
  - [x] Added `games-dir` to `KNOWN_KEYS`.
  - [x] Automatically call `initialize_library_structure` whenever `library-root` or `games-dir` is saved.
- [x] **Library Scanning & In-Slug Manifests (`core::library`)**:
  - [x] Implemented `detect_engine_from_path` for archive parent paths (`downloads/<engine>/<archive>`).
  - [x] Auto-create library structure at the start of `scan_games_dir`.
  - [x] Inspect both engine folders and direct game folders under `installed/` and `lzapps/`.
  - [x] Target archives to `installed/<engine>/<slug>/`.
  - [x] Save `app.json` inside the game slug folder alongside the extracted game files.
  - [x] Updated `list_installed` to scan engine subfolders (`installed/*/*/app.json`).
- [x] **Universal Launch Resolution (`core::launch`)**:
  - [x] Implemented `locate_manifest` to find games by slug across `list_installed` and root fallback.
- [x] **Multi-Format Archive Engine (`core::extract`)**:
  - [x] In-app byte progress for `.zip` and fallback for `.7z`, `.rar`, `.tar.*`, SFX `.exe`.
  - [x] HTML disguised as `.zip` detection.
  - [x] Collision-free extraction into `<engine>/<slug>/`.
- [x] **Work Queue Cleanup**:
  - [x] Cleared completed items from `TODO.md`.
  - [x] Cleared completed items from `ROADMAP.md`.
  - [x] Cleared resolved bugs from `BUGS.md`.
- [x] **Profile Icon Removal**:
  - [x] Removed unused avatar button and its CSS rules from the custom title bar in `src/routes/+layout.svelte`.
  - [x] Updated frontend unit tests in `src/lib/__tests__/page.test.ts` to assert profile button is absent.
- [x] **Extraction Progress Tracker & Queue Controls (`src/routes/downloads/+page.svelte`, `core::queue`, `core::extract`, `core::download`)**:
  - [x] Render live progress bar, percentage, and byte tracker for extraction jobs (`Extracting` status) alongside downloading jobs.
  - [x] Styled extraction progress bar fill with dedicated amber-to-accent gradient (`.bar-fill.extracting`).
  - [x] Added action buttons to cancel (`Cancel`) or delete (`✕`) queued and active items (`queued`, `resolving`, `downloading`, `dispatching`, `extracting`).
  - [x] Allowed deleting finished jobs (`dispatched`, `completed`, `failed`) and included `completed` in `hasFinishedJobs` for "Clear Finished".
  - [x] Backed cancellation in `core::queue`: aborting extraction and download streams early via error-propagating progress callback so background workers immediately stop on cancellation or deletion without corrupting job state.
  - [x] Added unit tests in `src/core/queue.rs` for cancelling, deleting, and aborting during progress callbacks.
  - [x] Updated frontend unit tests in `page.test.ts` to test extraction progress rendering and active/queued cancel and delete actions.
- [x] **Verification**:
  - [x] `cargo fmt --check` (clean, 0 diffs)
  - [x] `cargo clippy -- -D warnings` (clean, 0 warnings)
  - [x] `cargo check` (passes cleanly)
  - [x] `cargo test` (all 182 backend tests passed 100%)
  - [x] `npm run check` (0 errors, 0 warnings)
  - [x] `npm run test` (all 30 Vitest frontend tests passed 100%)
  - [x] Verified `G:\LewdZone` on disk: all 17 engine subfolders under `downloads/` and `installed/` created and active, plus `installed\renpy\harem-hotel` slug folder initialized.
