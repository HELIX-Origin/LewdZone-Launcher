# LewdZone Launcher — Repo Roadmap

**This document is the complete, authoritative roadmap for the entire
repository — from the very first commit onward.**

> **Accuracy contract:** ROADMAP.md must always be 100% accurate. It maps every
> feature we are working on. When a feature is abandoned, it is **removed** from
> this file (never marked "cancelled" and left dangling). When a feature is
> done, it moves to the right list. There is no history kept here — the git log
> is the history. Current status is in the **Done** and **Now** sections.
> The canonical tracked copy mirrors this file in the roadmap-first GitHub
> issue (Rule 04).

**Central command:** `🗺️ LewdZone App Build Roadmap` — the single living
tracking issue on GitHub. This file is its in-repo twin. Items here move
together; the issue body is updated when this file changes.

---

## North Star

A cross-platform desktop **game launcher** for lewdzone.com: browse a
Steam-like store, queue downloads, organize files, and create native shortcuts.
It is a **Tauri 2 app** (Rust + OS webview, Svelte frontend) whose binary also
exposes a **native Rust CLI**. The GUI and the CLI are two entry points into
the same Rust core: the same functions back both. The app is the primary
product; the CLI is scriptable standalone.

## Overview

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

---

## Done ✅

Everything shipped to `main`. (Initial push: `5d65506`.)

### Phase 0 — Foundation
- [x] `.agents/` agent ecosystem — 11 families, rules 00–13, skills, wiki
- [x] GitHub repo `HELIX-Origin/LewdZone-Launcher` + initial push
- [x] Root docs: `AGENTS.md`, `wiki/` (17 pages), `.gitignore`
- [x] `ROADMAP.md`, `TODO.md`, `BUGS.md` created (this trio)

## Now 🚧

Currently worked. **No sub-issues have been created on GitHub yet** — the
roadmap issue is the umbrella; sub-issues come from these items.

### Phase 1 — Diagnostics & Architecture
- [x] Scraping proof: archive pagination scheme — **`/games/page/N/`** pretty
  permalinks, 20 games/page, ~1146 pages; query `?page=N` ignored (normalized
  to base); filters combine as `?platform=PC&sort=New+to+Old` on the page path
- [ ] Mermaid compliance cleanup across `.agents/agents/*` (Rule 09)
  - systems-designer subgraph quoting; gui/testing subgraph quoting;
    dm-detector/folder-organizer label quoting
- [x] `src-tauri/src/` Rust crate scaffold + module boundaries (Rule 03)
- [x] ADRs for cross-layer contracts
  - [x] ADR-0001 — two entry points, one Rust core (native CLI + Tauri GUI)
  - [x] ADR-0002 — download-manager adapter layer (FDM/IDM/torrent)
  - [x] ADR-0003 — SQLite persistence (tokens, not URLs)
  - [x] ADR-0004 — content-provider enrichment layer
- [x] Config + SQLite bootstrap (`src-tauri/src/db.rs`, WAL schema, forward-only
  migrations + `schema_migrations`, DB tests green)
- [x] Content-provider layer design (`.agents/agents/content/` — provider
  contract, registry, six v1 providers) — design done, implementation pending

## Later ⏳

### Phase 2 — Core Implementation
- [ ] Scraper: catalog, game page, versions, download entries, genre tags
- [ ] Resolver: go-token `start` → `reveal`, retry, `\r` strip
- [ ] DM adapters: FDM, IDM, uTorrent/BitTorrent + detector + folder-organizer
- [ ] Job queue + sync pipeline (incremental, rate-limited)
- [ ] CLI commands: `sync/search/info/download/list/settings/shortcuts/launch/dm`
  with `--json` machine contract (progress on stderr)

### Phase 3 — Test Suite & Regression
- [ ] Rust unit/integration suites (cargo test) in `src-tauri/` + Svelte Vitest
      suites in `src/`, test helper modules in `src-tauri/tests/`
- [ ] App/CLI parity tests — same core commands drive both entry points
- [ ] Coverage floors: 85% overall, ~90% core, ~70% gui

### Phase 4 — Desktop App & Shortcuts
- [ ] Tauri 2 shell: Rust core, window lifecycle, shared-core commands
- [ ] Svelte views: Store / Library / Downloads / Settings (incl. API-keys
      section pasting content-provider keys)
- [ ] Native shortcuts (.lnk / .desktop / .app) + artwork from the
      content-provider layer (SteamGridDB icons, multi-provider covers)
- [ ] Content providers impl: VNDB, IGDB, itch.io, Steam Storefront, IndieDB
- [ ] Packaging: MSI+NSIS, .app+DMG, AppImage+deb+rpm, updater

### Phase 5 — Verification & Release
- [ ] Release gate green (cargo fmt/clippy/test, svelte-check, vitest, build smoke)
- [ ] Docs synced back to `wiki/`; release notes; tag `v0.1.0` / `v1.0.0`

## Abandoned 🗑️

*Nothing abandoned yet. When a feature is dropped, delete it from all sections
above (and from the tracking issue) — do not leave a corpse.*

---

## Acceptance Criteria (project-level)

- [ ] `lewdzone-launcher --help` clean on PowerShell **and** bash
- [ ] `lewdzone-launcher download --game treasure-of-nadia --version latest --platform PC --tab official --manager fdm --json` resolves and dispatches to a real DM
- [ ] Store/Library/Downloads/Settings pages all work; every action maps 1:1 to a CLI command
- [ ] Torrent links only accepted by a torrent-capable manager
- [ ] Files land in `<DownloadRoot>/Games/<Title>/<Title> - <Version> - <Platform>.ext`
- [ ] Full test suite + coverage floors green on CI

## Related

- [AGENTS.md](AGENTS.md) — operating manual
- [TODO.md](TODO.md) — fine-grained work queue (next steps)
- [BUGS.md](BUGS.md) — open bugs only
- [wiki](wiki/Home) — living documentation suite