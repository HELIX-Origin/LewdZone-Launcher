# LewdZone Launcher Changelog

Historical record of every change to the repository. Each release anchors to a
tag URL; commit entries link to their full commit. Newer releases are added at
the top; the current development state lives under `Unreleased`.

---

## Unreleased

### Added

- **docs(repo): add AGENTS operating manual, ROADMAP, TODO, and BUGS trackers** [9673193](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/967319311bae99e61a5965f0723169e7672e5e33)
- **docs(repo): scaffold lewdzone-launcher with .agents ecosystem, wiki, and gitignore** [5d65506](https://github.com/HELIX-Origin/LewdZone-Launcher/commit/5d655068a11c0b84b3de6e0c70c3589eb7050093)

---

| Component | Description | Status |
| :--- | :--- | :--- |
| Agent ecosystem | 10 families, rules 00–13, skills, dev templates | ✅ |
| Wiki suite | 16 pages, GitHub-wiki synced, relative links | ✅ |
| Root docs | `AGENTS.md`, `ROADMAP.md`, `TODO.md`, `BUGS.md` | ✅ |
| Roadmap tracking | Tracking issue #1, roadmap-first (Rule 04) | ✅ |
| Core implementation | Scraping, resolver, DM adapters, CLI engine | ⏳ Phase 1–2 |
| Desktop app | Tauri 2 shell + Store/Library/Downloads/Settings | ⏳ Phase 4 |
| Tests | vitest-style suite, fakes, coverage floors | ⏳ Phase 3 |
| Release | First tagged release (SemVer, Rule 08) | ⏳ Phase 5 |

## Additional

- **Conventions:** commits follow the emoji/type/scope guide (Rule 04); mermaid
  diagrams follow Rule 09; releases follow Rule 08.
- **Where things are:** living spec in `.agents/rules/index.md`, operating manual
  in `AGENTS.md`, roadmap in `ROADMAP.md`, open bugs in `BUGS.md`, fine-grained
  work queue in `TODO.md`.
- **Format guide for this file:** one release section per tag; change categories
  (`Added` / `Removed` / `Fixed` / etc.); commit list with short-hash links;
  summary table with status icons; extra notes under `Additional`.