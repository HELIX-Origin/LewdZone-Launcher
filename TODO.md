# TODO (work queue)

Living work queue. Items move to ROADMAP.md phases once they get sub-issues;
checked items go to `git log`. Add new items here first, then pull into the
roadmap when they're scoped.

## Immediate

- [x] Rename `PLAN.md` → `ROADMAP.md`
- [x] Fix mermaid compliance defects in `.agents/agents/*` (Rule 09) — still to verify in cleanup pass
- [x] Fix `rule-04` / `rule-08` emoji mojibake (done via a one-off scanner test)
- [x] Update `rule-04` scopes: `fdm` → `dm` (+ Sub-Issue 2 "DM wiring")
- [x] `rule-01` module list: `fdm` → `dm`
- [x] `rule-08` example title: `catalog sync + FDM queue` → `catalog sync + DM queue`
- [x] `rule-00-governance` family list: `database/fdm/cli` → `database/dm/cli`
- [ ] `testing.md` + `test-suite-architect.md` — swap `fdm` fake / `test_fdm` for dm family (verify)
- [ ] `module-contractor.md` — `T4 fdm bridge` → dm bridge (verify)
- [ ] `cli.md` / `architect.md` / `index.md` — finish `fdm`→`dm` refs (verify)
- [x] `launch-fdm` skill → DM-agnostic `launch-download` skill (done; `launch-fdm/` deleted)
- [ ] Add `gui-build-loop` + `package-desktop-app` skills (referenced by gui.md)
- [ ] Write templates layer: `agent`, `skill`, `rule`, `module-rust`, `test-rust`, `migration-sql`, `adr`, `issue`, `issue-roadmap`, `release-notes`, `commit-message-guide`, `shortcut-artwork`, Tauri `view`/`command`, `cli-json-contract`, `dm-adapter`
- [ ] Write `.agents/README.md` index
- [x] Scaffold `src-tauri/src/` Rust crate layout (Rule 03 skeleton)
- [x] Add `.gitignore` entry for `scratch/` (already in initial push)
- [ ] Verify archive pagination scheme (`?page=N` vs `/page/N/`) on live site
- [x] Promote scratch scanners into the Rust test suite (mojibake, fdm→dm stale refs, typo check)
- [x] Add `cargo test` config + coverage floors (Rule 11)
- [x] Add content-provider layer agents (`.agents/agents/content/`, 7 files) + provider-registry
- [x] Wire multi-provider artwork into shortcuts family (`artwork-fetch` dispatches via content registry)
- [x] Settings/API-keys section in view-designer + wiki (Configuration, Security, Getting-Started)
- [x] Add `wiki/Content-Providers.md` + sidebar + README/Home/Development/Agents/Architecture touchpoints
- [ ] Implement `src-tauri/src/content/` (contract, registry, adapter stubs) + `game_external`/`artwork_cache` schema
- [ ] Implement `enrich-game-and-art` skill + `content-provider` template
- [ ] `rule-03`/`rule-10` — fold content-provider keys/secrets wording (verify coverage)

## Backlog (unscoped)

- [ ] SteamGridDB artwork pipeline end-to-end via content layer (search → pick → ico → cache)
- [ ] Enrichment e2e for a thin title (VNDB description + SteamGridDB icon + VNDB cover, cached + offline replay)
- [ ] Per-OS shortcut builders (.lnk / .desktop / macOS alias) tested
- [ ] Perf budgets: cold start <2s, list <300ms, search <200ms, parse <400ms
- [ ] CI workflows: cargo fmt/clippy/test, svelte-check, vitest; tauri build matrix