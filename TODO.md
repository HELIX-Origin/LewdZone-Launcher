# TODO (work queue)

Living work queue. Items move to ROADMAP.md phases once they get sub-issues;
checked items go to `git log`. Add new items here first, then pull into the
roadmap when they're scoped.

## Immediate

- [ ] Rename `PLAN.md` → `ROADMAP.md` (done — this file)
- [ ] Fix mermaid compliance defects in `.agents/agents/*` (Rule 09):
  - [ ] `systems-designer.md` — unquoted subgraph titles + `lewzodone.com` typo
  - [ ] `gui.md` / `testing.md` — unquoted subgraphs
  - [ ] `dm-detector.md` / `folder-organizer.md` — unquoted backslash / angle labels
- [ ] Fix `rule-04-remote-issue-protocol.md` emoji mojibake (ðŸ — corrupted arrows/emoji tables)
- [ ] Update `rule-04` scopes list: `fdm` → `dm` (and Sub-Issue 2 "FDM wiring" → "DM wiring")
- [ ] `rule-01` subpackage list: `fdm` → `dm`
- [ ] `rule-08` example title: `v1.4.2 — catalog sync + FDM queue` → DM wording
- [ ] `testing.md` + `test-suite-architect.md` — swap `fdm` fake / `test_fdm` for dm family
- [ ] `module-contractor.md` — `T4 fdm bridge` → dm bridge
- [ ] `cli.md` / `architect.md` / `index.md` — finish `fdm`→`dm` refs
- [ ] `launch-fdm` skill → DM-agnostic `launch-download` skill
- [ ] Add `gui-build-loop` + `package-desktop-app` skills (referenced by gui.md)
- [ ] Write templates layer: `agent`, `skill`, `rule`, `module-python`, `test-python`, `migration-sql`, `adr`, `issue`, `issue-roadmap`, `release-notes`, `commit-message-guide`, `shortcut-artwork`, Tauri `view`/`command`, `cli-json-contract`, `dm-adapter`
- [ ] Write `.agents/README.md` index
- [ ] Scaffold `src/lewdzone_launcher/` package layout (follows Rule 03 skeleton)
- [ ] Add `.gitignore` entry for `scratch/` (already in initial push? verify)
- [ ] Verify archive pagination scheme (`?page=N` vs `/page/N/`) on live site

## Backlog (unscoped)

- [ ] SteamGridDB artwork pipeline end-to-end (search → pick → ico → cache)
- [ ] Per-OS shortcut builders (win32com / .desktop / macOS alias) tested
- [ ] Perf budgets: cold start <2s, list <300ms, search <200ms, parse <400ms
- [ ] CI workflows: ruff/pyright/pytest/bandit/pip-audit; tauri build matrix