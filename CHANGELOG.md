# 📜 LewdZone Launcher Changelog

Historical record of every change to the repository. Each release anchors to a
tag URL; commit entries link to their full commit. Newer releases are added at
the top; the current development state lives under `Unreleased`.

---

## ⏳ Unreleased

*Nothing yet.*

---

## [v0.2.0](https://github.com/HELIX-Origin/LewdZone-Launcher/releases/tag/v0.2.0) — 2026-09-23

### ✨ Added

- **feat(download): replace download-manager layer with in-app streaming and OS-native dispatch** [8c691f9](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/8c691f9)
- **feat(download): downloads/lzapps install layout, SQLite secrets, and agent cleanup** [b5b39e8](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/b5b39e8)
- **feat(library,launch): read lzapps app.json manifests, real launch, and GUI launch button** [71099dd](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/71099dd)
- **feat(queue): persist download queue to SQLite and resume on startup** [07d18f7](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/07d18f7)
- **feat(content): content-provider and artwork cache pipeline** [06477c2](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/06477c2)
- **feat(favorites): heart toggle on library tiles and real favorites page** [87c5869](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/87c5869)
- **feat(content,gui): proper genre support with external_genres field and LewdZone-default enrichment** [512cc92](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/512cc92)

### 🛠 Fixed

- **fix(content): use LewdZone scraped data as default metadata source before external providers** [cf36391](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/cf36391)
- **fix(favorites): create temp dir in test and remove invalid views column** [a08ca6a](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/a08ca6a)

### Removed

- Download-manager adapters (FDM, IDM, torrent) — replaced by OS-native pass-through.
- Android platform support in the desktop app.

---

## v0.1.0 — 2026-09-18

### ✨ Added

- **docs(repo): add AGENTS operating manual, ROADMAP, TODO, and BUGS trackers** [9673193](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/967319311bae99e61a5965f0723169e7672e5e33)
- **docs(repo): scaffold lewdzone-launcher with .agents ecosystem, wiki, and gitignore** [5d65506](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/5d655068a11c0b84b3de6e0c70c3589eb7050093)

---

| Component | Description | Status |
| :--- | :--- | :--- |
| Agent ecosystem | 10 families, rules 00–13, skills, dev templates | ✅ |
| Wiki suite | 16 pages, GitHub-wiki synced, relative links | ✅ |
| Root docs | `AGENTS.md`, `ROADMAP.md`, `TODO.md`, `BUGS.md`, `CHANGELOG.md` | ✅ |
| Roadmap tracking | Tracking issue #1, roadmap-first (Rule 04) | ✅ |
| Core implementation | Scraping, resolver, in-app streaming, CLI engine | ✅ v0.2.0 |
| Desktop app | Tauri 2 shell + Store/Library/Downloads/Favorites/Settings | ✅ v0.2.0 |
| Tests | cargo test 177, Vitest 29, clippy/fmt/svelte-check green | ✅ v0.2.0 |
| Release | Tagged releases (SemVer, Rule 08) | ✅ v0.2.0 |

## Additional

- **Conventions:** commits follow the emoji/type/scope guide (Rule 04); mermaid
  diagrams follow Rule 09; releases follow Rule 08.
- **Where things are:** living spec in `.agents/rules/index.md`, operating manual
  in `AGENTS.md`, roadmap in `ROADMAP.md`, open bugs in `BUGS.md`, fine-grained
  work queue in `TODO.md`.
- **Format guide for this file:** one release section per tag; change categories
  (`Added` / `Removed` / `Fixed` / etc.); commit list with short-hash links;
  summary table with status icons; extra notes under `Additional`.