# 📋 TODO (work queue)

Living work queue. Items move to ROADMAP.md phases once they get sub-issues;
checked items go to `git log`. Add new items here first, then pull into the
roadmap when they're scoped.

## 🚧 Immediate

(No open blockers. All core download, resolver, storefront, and queue features implemented and tested.)

## 📦 Recently Completed

### Downloads, Resolver & Storefront Enhancements
- [x] **Ad-Free Child Resolver Window**: Built custom in-app redirect page (`/resolver`) with clean countdown and API negotiation, completely preventing third-party ad injection/click hijacking.
- [x] **Corrupted Zip / Missing EOCD Fix**: Reject `text/html` in `download_stream` and inspect zip magic bytes (`PK\x03\x04`) in `extract_zip` with clear error diagnostics.
- [x] **LewdZone Per-Source Download Layout**: Replaced 4-dropdown selector with authentic LewdZone version selection, official/community tabs, and platform/variant groupings.
- [x] **Download Provider Brand Icons**: Added authentic branded provider icons for all supported cloud hosts on download buttons.
- [x] **Download Queue Management**: Added cancellation for in-flight jobs, individual deletion (`✕`) for completed/failed jobs, and "Clear Finished" header action.
- [x] **Game Page Image Preview Carousel**: Enhanced screenshot extraction for full-res and carousel images, with previous/next controls, thumbnail navigation, and fullscreen lightbox.
- [x] **Settings Cleanup**: Removed redundant preferred sources setting since users choose directly from per-source buttons.

### Architecture & Persistence (Memory-First)
- [x] **Memory-First Cache Layer**: Added `game_cache`, `page_cache`, and `genres_cache` in `AppState` for instant (0ms) memory lookups.
- [x] **SQLite Persistence Integration**: Database fallback when cache misses, automatic upsert to SQLite upon network fetch, and full game model reconstruction in `db::repo::game_by_slug`.
- [x] **Store Detail Page Deduplication**: Removed concurrent double-loads and redundant in-flight fetches in `src/routes/store/[slug]/+page.svelte`.
- [x] **Single-Version Fallback**: Preserved download entries for games without `#lz-version-select` dropdowns.
- [x] **Removal of External Providers (ADR-0006)**: Removed stalling external metadata providers (IGDB, VNDB, Steam, SteamGridDB, itch, IndieDB) so store pages load immediately from scraped site metadata.
- [x] **Discontinued Desktop Shortcuts (ADR-0006)**: Abandoned desktop shortcut creation due to official site lacking fitting square icon assets; cleaned up Library UI and CLI handlers.

### `.agents` cleanup (fdm → dm family)
- [x] Rename `PLAN.md` → `ROADMAP.md`
- [x] Fix mermaid compliance defects in `.agents/agents/*` (Rule 09)
- [x] Fix `rule-04` / `rule-08` emoji mojibake
- [x] Update `rule-04` scopes: `fdm` → `dm` (+ Sub-Issue 2 "DM wiring")
- [x] `rule-01` module list: `fdm` → `dm`
- [x] `rule-08` example title: `catalog sync + FDM queue` → `catalog sync + DM queue`
- [x] `rule-00-governance` family list: `database/fdm/cli` → `database/dm/cli`
- [x] `launch-fdm` skill → DM-agnostic `launch-download` skill (`launch-fdm/` deleted)
- [x] Delete stale `.agents/agents/fdm/fdm.md` shim
- [x] `testing.md` + `mock-engineer.md` — swap remaining `fdm` fake / `test_fdm` wording for the dm family
- [x] `module-contractor.md` — `T4 fdm bridge` → dm bridge
- [x] Write `.agents/README.md` index and templates layer
- [x] Add `gui-build-loop` + `package-desktop-app` skills
- [x] Verify archive pagination scheme (`/games/page/N/`) on live site

### Core / GUI features
- [x] App relayout to dark cyberpunk spec: left icon sidebar + top header, deep cyan/charcoal gradient bg, neon accents, glassmorphism, thin scrollbar. Store is the main view.
- [x] Storefront: top search, left category/genre rail, hero + media-grid rows, tile → game detail.
- [x] Custom macOS-style title bar: frameless window (`decorations: false`) with traffic-light window controls + custom menu bar.
- [x] Download scheduler: `download-grace-seconds` pacing between dispatch starts.
- [x] **Non-blocking async download queue** (Rust core): `game_download` enqueues and returns instantly; background worker resolves + dispatches one request at a time.
- [x] Bundled theme skins (Nord / Dracula / Material) shipped in-repo + `home-page` launch tab setting.
- [x] **Downloads page**: poll `downloads_list` and render job status/progress.
- [x] Persist `download_job` rows + resume across restarts.
- [x] Library view: list installed titles from `lzapps/<slug>/app.json` with launch support.
- [x] Multi-format installs: infer actual game binary for launch from engine/tag taxonomy.
- [x] Favorites: SQLite-backed heart toggle on Library tiles + Favorites page listing.
- [x] App icons: generated from `assets/appicon.png` via `tauri icon` and wired into bundle config.

## 🗄️ Backlog (unscoped)

- [ ] Perf budgets: cold start <2s, list <300ms, search <200ms, parse <400ms (measurement deferred to post-v0.1.0 optimization pass).
- [x] CI workflows: cargo fmt/clippy/test, svelte-check, vitest; tauri build matrix.