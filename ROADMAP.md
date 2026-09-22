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

Cross-platform desktop game launcher for lewdzone.com: Steam-like Store
(browse/search/download), Library (icons, covers, descriptions), Downloads
(queue), Settings. **Single Tauri 2 binary** with two faces: a windowed GUI
(SvelteKit frontend in `src/`, Rust core in `src-tauri/`) and a native Rust CLI
(same core, clap commands, `--json` machine output). Internal folders mirror the
Steam client layout
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
- [x] Native Rust CLI core (clap): sync/search/info/download/list/settings/shortcuts/launch/dm + exit-code contract (0–5) + `--json`
- [x] Shared core: paths, settings, library (Steam-mirror), skins/theme system (ADR-0005) — Rust tests green
- [x] Tauri commands bridge: settings_get/settings_set/themes_list/themes_tokens/themes_apply
- [x] App shell: topbar nav (Store/Library/Downloads/Settings) + 4 placeholder views; frontend vitest suites green, svelte-check green
- [x] Toolchain (Rust 1.98 / MSVC via VS2026 Build Tools), cargo test + clippy + fmt green
- [x] Scraper family: archive card parser with `?page=N` pagination, game-page parser (versions, tabs, genres, screenshots), polite fetch with bounded retries
- [x] Resolver family: go-link token → start/reveal API → real URL, retries, host allowlist (Rule 10)
- [x] DM layer: FDM / IDM / torrent adapters, detection registry, folder organizer (ADR-0002)
- [x] `download` command: version/platform/tab selection + dispatch to the active manager
- [x] SQLite catalog: forward-only migrations, repo layer, resumable sync pipeline + prune (Rule 06, ADR-0003)
- [x] `list` + `info` CLI commands backed by the synced catalog
- [x] Docs/CLI drift cleanup: README, wiki, and `.agents` docs now match the real CLI (`dm <name>`, `list --library`/`--jobs`, no `--manager`/`--status`)

## Now 🚧 (Phase 2 — Storefront + Core)

- [ ] Fix any remaining verification gaps (final svelte-check pass, clippy/fmt clean)
- [ ] Storefront catalog view: real tiles + game-page lookup (search/content-provider layer)
- [ ] Download queue: persist `download_job` rows and wire resume/queue flow (the table exists; `download.rs` doesn't write jobs yet)
- [ ] Router recreation of store pages (ripped UI mirrored into local HTML)

## Later ⏳

### Phase 3 — Test Suite & Regression
- Frontend unit tests for all four views
- App/CLI parity tests + protocol tests
- Coverage floors: 85% overall, ~90% core, ~70% gui

### Phase 4 — Packaging & Shortcuts
- Native shortcuts (.lnk / .desktop / .app) + SteamGridDB artwork
- Packaging: MSI+NSIS, .app+DMG, AppImage+deb+rpm, updater
- **Releases page:** prebuilt installers per OS+arch
- **GitHub Packages (npm):** CLI-only package for headless/no-GUI users

### Phase 5 — Verification & Release
- Release gate (cargo test/clippy/fmt, vitest, svelte-check, security, build smoke)
- Docs sync → wiki, release notes, tag `v0.1.0` / `v1.0.0`

## ✅ Acceptance Criteria

- [ ] `lewdzone --help` clean on PowerShell and bash
- [ ] `download --game treasure-of-nadia --json` resolves and dispatches to the active manager (`lewdzone dm fdm` selects it)
- [ ] Store/Library/Downloads/Settings all map 1:1 to an invoke command or CLI command
- [ ] Torrent links only accepted by a torrent-capable manager
- [ ] Files land in `<Library>/common/<Title>/` per ADR-0005 mirror layout
- [ ] Rust + frontend suites green on CI

## 🔗 Related

In-repo: `TODO.md`, `BUGS.md`, `AGENTS.md`, wiki pages (Architecture,
CLI-Reference, Download-Managers, Development). Living spec: `.agents/rules/index.md`.
ADRs: `.agents/adr/` (storage/settings patterns).