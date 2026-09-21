# LewdZone Launcher — Agent Operating Manual

The LewdZone Launcher is a cross-platform desktop game launcher that scrapes
[lewdzone.com](https://lewdzone.com), resolves download tokens, and hands
resolved URLs to an installed download manager (FDM, IDM, or a torrent client).
It is a **Tauri 2 app** (Rust + OS webview, Svelte frontend) that drives a
**Python CLI sidecar** over JSON/JSONL subprocess communication. The app is the
primary product; the CLI is the engine (also scriptable standalone).

> **App-first rule:** every GUI action maps 1:1 to a CLI command. The app never
> imports the Python package. Logic lives in the CLI, not in the webview.

---

## Agent Catalog

| Family | Primary | Sub-agents | Scope |
| --- | --- | --- | --- |
| `architect` | architect.md | systems-designer, module-contractor | System design, module topology, ADRs, contract locking |
| `scraper` | scraper.md | archive-scraper, game-page-scraper, fixture-engineer | lewdzone.com page scraping, fixtures, HTML parsing |
| `resolver` | resolver.md | token-prober, dispatch-builder | Go-link token resolution via start→reveal API |
| `database` | database.md | schema-designer, sync-orchestrator | SQLite schema, migrations, sync pipeline |
| `dm` | dm.md | dm-detector, fdm-adapter, idm-adapter, torrent-adapter, folder-organizer | Download-manager detection, per-manager dispatch, folder folding |
| `cli` | cli.md | command-designer, output-formatter | CLI engine, commands, JSON/JSONL contract |
| `gui` | gui.md | app-shell, view-designer, sidecar-driver | Tauri 2 desktop app, Rust core, Svelte views, sidecar protocol |
| `shortcuts` | shortcuts.md | artwork-fetch, shortcut-builder | SteamGridDB artwork, per-OS native shortcuts |
| `content` | content.md | provider-registry, steamgriddb-provider, vndb-provider, igdb-provider, itch-provider, steam-provider, indiedb-provider | External info + art enrichment (VNDB, IGDB, Steam, itch.io, IndieDB) |
| `testing` | testing.md | fixture-crafter, mock-engineer, test-suite-architect, debugger | Test layers, fakes, coverage floors |
| `review` | review.md | security-auditor, perf-auditor | Gate pipeline, threat modeling, perf budgets |

See [wiki/Agents](wiki/Agents) for the full agent catalog with sub-agent details.

---

## Rules

| # | Name | Scope |
| --- | --- | --- |
| 00 | Governance & Delegation | Command chain, one-owner-per-artifact |
| 01 | Code Style (Python) | Ruff, Pyright strict, 88-char, double quotes |
| 02 | Naming Conventions | Canonical vocab, file naming, enums |
| 03 | Module Architecture | src layout, import-linter layers, contracts |
| 04 | Remote Issue Protocol | Roadmap-first, sub-issue lifecycle, commit format |
| 05 | Network Etiquette | 1 req/s, retry, offline fixtures, live opt-in |
| 06 | SQLite Conventions | WAL, FK, migrations, store tokens not URLs |
| 07 | Download Manager Integration | Adapter contract, cross-platform spawn |
| 08 | Release Standards | SemVer, version sync, verification gate |
| 09 | Mermaid Standards | GitHub v10, quoted labels, ≤12 nodes |
| 10 | Security | Secrets, allowlists, subprocess safety |
| 11 | Testing | Unit/integration/live layers, 85% coverage |
| 12 | Error Handling | Exit codes 0–5, typed errors, stdout vs stderr |
| 13 | GUI Conventions | Tauri subprocess model, parity guardrails |

Full descriptions: `.agents/rules/index.md` and `wiki/Design-Conventions`.

---

## Verification

Before any merge to `main`, all of the following must pass:

```bash
# Formatting & types
ruff format --check src/ tests/
ruff check src/ tests/
pyright

# Tests
python -m pytest -q
python -m pytest --cov --cov-fail-under=85

# Security
bandit -r src/
pip-audit

# CLI smoke
lewdzone-launcher --version
lewdzone-launcher --help
```

For the Tauri desktop app (when scaffolded):

```bash
npm run tauri build     # all platforms
```

---

## Governance Principles

1. **One owner per artifact.** A file or module has exactly one owning agent.
2. **CLI is the primary interface.** The app is a thin Tauri shell around the CLI.
3. **Rules before code.** No implementation starts without its governing rule.
4. **Fail loudly.** Never swallow errors to keep the build green.
5. **ADR before contract change.** Any cross-layer interface change needs an ADR
   filed in `docs/adr/` before implementation begins.

See [Rule 00: Governance & Delegation](.agents/rules/rule-00-governance.md)
for the full command chain and delegation matrix.