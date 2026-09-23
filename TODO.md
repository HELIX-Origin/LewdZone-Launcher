# 📋 TODO (work queue)

Living work queue. Items move to ROADMAP.md phases once they get sub-issues;
checked items go to `git log`. Add new items here first, then pull into the
roadmap when they're scoped.

## 🚧 Immediate

### `.agents` cleanup (fdm → dm family)

- [x] Rename `PLAN.md` → `ROADMAP.md`
- [x] Fix mermaid compliance defects in `.agents/agents/*` (Rule 09) — still to verify in cleanup pass
- [x] Fix `rule-04` / `rule-08` emoji mojibake (done via a one-off scanner test)
- [x] Update `rule-04` scopes: `fdm` → `dm` (+ Sub-Issue 2 "DM wiring")
- [x] `rule-01` module list: `fdm` → `dm`
- [x] `rule-08` example title: `catalog sync + FDM queue` → `catalog sync + DM queue`
- [x] `rule-00-governance` family list: `database/fdm/cli` → `database/dm/cli`
- [x] `launch-fdm` skill → DM-agnostic `launch-download` skill (done; `launch-fdm/` deleted)
- [x] Delete stale `.agents/agents/fdm/fdm.md` shim (the family is `dm`; `fdm-adapter` lives under `.agents/agents/dm/`)
- [x] `testing.md` + `mock-engineer.md` — swap remaining `fdm` fake / `test_fdm` wording for the dm family
- [x] `module-contractor.md` — `T4 fdm bridge` → dm bridge
- [x] Write `.agents/README.md` index
  - [ ] Write the rest of the templates layer (only `adr.md` + `changelog.md` exist of the ~17 planned)
  - [x] Add `gui-build-loop` + `package-desktop-app` skills (referenced by gui.md)
  - [x] Verify archive pagination scheme (`?page=N` vs `/page/N/`) on live site

### Content-provider layer (Phase 2 engine)

  - [x] Implement content-provider contract, registry, and providers (SteamGridDB, VNDB, IGDB, itch.io, Steam, IndieDB) + `game_external`/`artwork_cache` schema
  - [x] LewdZone scraped data is the default metadata source; external providers fill missing/enhanced fields only
  - [x] Proper genre support: `external_genres` distinct from LewdZone tags
  - [ ] Implement `enrich-game-and-art` skill + `content-provider` template
  - [x] `rule-03`/`rule-10` — fold content-provider keys/secrets wording (verify coverage)

### Core / GUI features

- [x] App relayout to the dark cyberpunk spec: left icon sidebar + top header (wide rounded search + circular profile avatar), deep cyan/charcoal gradient bg, neon accents, glassmorphism, thin scrollbar. **Home is not a separate tab** — it is the Store.
- [x] Store view = storefront: top search, left category/genre rail, hero + media-grid rows (~9 poster-ratio tiles 2:3/3:4), tile → game detail. Backed by live search (`?s=`), genre pages, platform/sort filters (headless `/games/` + `/game-genre/`).
- [x] Custom macOS-style title bar: frameless window (`decorations: false`) with traffic-light window controls (red/yellow/green) + custom menu bar (File/View/Help menus, profile button).
- [x] Download-source controls: `source-priority` reordering + per-game `game_sources` panel (preferred default).
- [x] Download scheduler: `download-grace-seconds` pacing between dispatch starts (free-tier throttle protection).
- [x] **Non-blocking async download queue** (Rust core): `game_download` enqueues and returns instantly; a background worker resolves + dispatches one request at a time; `downloads_list` exposes progress. (Pixeldrain proxy-cycle "bypass" was implemented then removed — the upstream service is dead.)
- [x] Bundled theme skins (Nord / Dracula / Material) shipped in-repo + `home-page` launch tab setting.
- [x] **Downloads page**: poll `downloads_list` and render each job's status/message. Add Downloads to the icon sidebar nav. Update the store detail `download()` to the queued (`QueueJob`) return.
- [x] Persist `download_job` rows + resume across restarts.
- [x] Library view = downloaded games: list installed titles from `lzapps/<slug>/app.json`, with launch support.
- [x] Multi-format installs: the site ships games as web HTML, `.exe`, and other formats; use the site's engine/tag taxonomy to infer the actual game binary for launch (engine → binary discovery).
- [x] Favorites: SQLite-backed heart toggle on Library tiles + Favorites page listing.
- [x] Clickable Store tiles: tile/title navigate to `/store/<slug>`.
- [x] App icons: regenerate from `assets/appicon.png` via `tauri icon`, wire the outputs into `tauri.conf.json` (`bundle.icon`) and fix the non-rendering sidebar logo image.

## 🗄️ Backlog (unscoped)

- [ ] SteamGridDB artwork pipeline end-to-end via content layer (search → pick → ico → cache)
- [ ] Enrichment e2e for a thin title (VNDB description + SteamGridDB icon + VNDB cover, cached + offline replay)
- [ ] Per-OS shortcut builders (.lnk / .desktop / macOS alias) tested
- [ ] Perf budgets: cold start <2s, list <300ms, search <200ms, parse <400ms
- [ ] CI workflows: cargo fmt/clippy/test, svelte-check, vitest; tauri build matrix