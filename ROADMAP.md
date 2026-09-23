# 🗺️ LewdZone App Build Roadmap

**This is the complete, authoritative roadmap for the repository.** Living plan:
edited in place as work progresses — a tracked twin of the roadmap tracking
[issue #1](https://github.com/HELIX-Origin/lewdzone/issues/1)
([Rule 04](.agents/rules/rule-04-remote-issue-protocol.md)).

> **Accuracy contract:** must always be 100% accurate. When a feature is
> abandoned it is **removed** here and from the roadmap issue — never left as a
> cancelled corpse. When done, it moves to **Done**. History lives in the git
> log, not here.

## 🧭 North Star

Cross-platform desktop game launcher for lewdzone.com: Storefront
(browse/search/download), Library (icons, covers, descriptions), Downloads
(queue), Settings. **Single Tauri 2 binary** with two faces: a windowed GUI
(SvelteKit frontend in `src/`, Rust core in `src-tauri/`) and a native Rust CLI
(same core, clap commands, `--json` machine output). Internal folders mirror a
classic launcher layout
([ADR-0005](.agents/adr/0005-steam-mirror-folder-structure.md)).

## 🗓️ Plan

```mermaid
flowchart TD
    ROAD["roadmap tracking issue"]
    ROAD --> P1["Sub-Issue 1: diagnostics + architecture"]
    ROAD --> P2["Sub-Issue 2: core implementation"]
    ROAD --> P3["Sub-Issue 3: test suite + regression"]
    ROAD --> P4["Sub-Issue 4: verification + docs sync"]
    P1 --> P2
    P2 --> P3
    P3 --> P4
    style ROAD fill:#e11,color:#fff
```

## Done ✅

- [x] `.agents/` ecosystem — 10 families, rules 00–13, skills, wiki, ADRs 0001–0005
- [x] GitHub repo + initial push
- [x] Root docs: `AGENTS.md`, wiki (16 pages), legal pages (LICENSE/PRIVACY/TOS/SECURITY)
- [x] Restructure: Python CLI deleted; repo root IS the Tauri 2 app (`src/` + `src-tauri/`)
- [x] Native Rust CLI core (clap): sync/search/info/download/list/settings/shortcuts/launch/favorites + exit-code contract (0–5) + `--json`
- [x] Shared core: paths, settings, library, skins/theme system (ADR-0005) — Rust tests green
- [x] Tauri commands bridge: settings_get/settings_set/themes_list/themes_tokens/themes_apply
- [x] App shell: topbar nav (Store/Library/Downloads/Settings) + 4 placeholder views; frontend vitest suites green, svelte-check green
- [x] Toolchain (Rust 1.98 / MSVC via VS2026 Build Tools), cargo test + clippy + fmt green
- [x] Scraper family: archive card parser with `?page=N` pagination, game-page parser (versions, tabs, genres, screenshots), polite fetch with bounded retries
- [x] Resolver family: go-link token → start/reveal API → real URL, retries, host allowlist (Rule 10)
- [x] Download dispatch: version/platform/tab selection; direct-file hosts stream in-app, others open via OS default handler
- [x] SQLite catalog: forward-only migrations, repo layer, resumable sync pipeline + prune (Rule 06, ADR-0003)
- [x] `list` + `info` CLI commands backed by the synced catalog
- [x] Docs/CLI drift cleanup: README, wiki, and `.agents` docs now match the real CLI (`download`, `list --library`/`--jobs`, no `--manager`/`--status`)

## Now 🚧 (Phase 2 — Storefront + Core)

- [x] Fix any remaining verification gaps (final svelte-check pass, clippy/fmt clean)
- [x] **GUI relayout** to the dark cyberpunk spec: left icon sidebar (Store/Favorites/Library/Settings; Downloads pending its page) + top header (rounded search + profile avatar), deep cyan/charcoal gradient, neon accents, glassmorphism, thin scrollbar. **Home is not a separate tab** — it is the Store.
- [x] **Storefront**: top search, left genre/category rail, hero + media-grid rows (~9 poster tiles 2:3/3:4), tile → game detail. Live search (`?s=`), genre pages, platform/sort filters (`/games/` + `/game-genre/`). Backed by the full site filter surface (q/platform/engine/state/sort/tags[]/tags-exclude[]) and no ads / no redirect exposure (downloads resolve via in-app stream or OS-native dispatch).
- [x] **Library = downloaded games**: list installed titles from `lzapps/<slug>/app.json`; launch support from the Library view.
- [x] **Multi-format launch**: games ship as web HTML / `.exe` / other formats; infer the game binary via the site's engine/tag taxonomy so Launch opens the right target.
- [x] Custom macOS-style title bar: frameless window (`decorations: false`) + traffic lights (red/yellow/green) + custom File/View/Help menus + profile button. 29 frontend tests green.
- [x] Download-source controls: `source-priority` reordering + per-game `game_sources` panel (preferred default).
- [x] Download scheduler: `download-grace-seconds` pacing between dispatch starts (free-tier throttle protection). Pixeldrain proxy-cycle "bypass" was implemented then removed — the upstream service is dead.
- [x] **Non-blocking async download queue** (Rust core): `game_download` enqueues and returns instantly; a background worker resolves + dispatches one request at a time; the Downloads view polls `downloads_list`.
- [x] **Downloads page**: poll `downloads_list` and render each job's status/message; add Downloads to the icon sidebar nav; store detail `download()` returns `queued` feedback.
- [x] Persist `download_job` rows + resume across restarts.
- [x] Storefront catalog view: real tiles + game-page lookup; clickable tiles navigate to detail.
- [x] Content-provider layer: artwork cache + SteamGridDB, VNDB, IGDB, itch.io, Steam, IndieDB providers; LewdZone scraped data is the default metadata source.
- [x] Proper genre support: `external_genres` from providers, distinct from LewdZone tags.
- [x] App icons: regenerate from `assets/appicon.png` via `tauri icon`, wire into `tauri.conf.json` bundle icons, and fix the non-rendering sidebar logo image.
- [x] Favorites: SQLite-backed heart toggle on Library tiles + Favorites page.
- [x] `.agents/` rule-03/rule-10 content-provider wording + `gui-build-loop` + `package-desktop-app` skills.
- [x] Live archive pagination verification (`/games/page/N/`).
- [x] SteamGridDB artwork end-to-end + enrichment e2e on game detail page (description, screenshots, rating, external_genres).
- [x] Per-OS native shortcuts (.lnk / .desktop / .app) with Library button + CLI command.

## Later ⏳

### Phase 3 — Test Suite & Regression
- Frontend unit tests for all four views
- App/CLI parity tests + protocol tests
- Coverage floors: 85% overall, ~90% core, ~70% gui

### Phase 4 — Packaging & Shortcuts
- Native shortcuts (.lnk / .desktop / .app) + SteamGridDB artwork
- Packaging: MSI+NSIS, .app+DMG, AppImage+deb+rpm, updater
- **Releases page:** prebuilt installers per OS+arch (no npm/GitHub Packages publishing — the CLI ships inside the app bundle)

### Phase 5 — Verification & Release
- Release gate (cargo test/clippy/fmt, vitest, svelte-check, security, build smoke)
- Docs sync → wiki, release notes, tag `v0.1.0`

## ✅ Acceptance Criteria

- [ ] `lewdzone --help` clean on PowerShell and bash
- [x] `download --game treasure-of-nadia --json` resolves and dispatches (direct-file hosts stream in-app; others open via OS default handler)
- [x] Store/Library/Downloads/Settings all map 1:1 to an invoke command or CLI command
- [x] Files land in `<downloads>/Games/<Title>/` staging and `<lzapps>/<slug>/` installs
- [ ] Rust + frontend suites green on CI

## 🔗 Related

In-repo: `TODO.md`, `BUGS.md`, `AGENTS.md`, wiki pages (Architecture,
CLI-Reference, Download-Managers, Development). Living spec: `.agents/rules/index.md`.
ADRs: `.agents/adr/` (storage/settings patterns).