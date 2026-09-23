# LewdZone Launcher — Agent Operating Manual

The LewdZone Launcher is a cross-platform desktop game launcher that scrapes
[lewdzone.com](https://lewdzone.com), resolves download tokens, streams
direct-file hosts in-app with byte progress, and hands every other resolved
URL to the OS default handler (the installed cloud app or the browser) — no
download manager needed.
It is a **Tauri 2 app** (Rust + OS webview, Svelte frontend) whose binary also
exposes a **native Rust CLI** (`src-tauri/src/cli.rs`). The GUI and the CLI are
two entry points into the same Rust core: the same functions back both (Rule
03, Rule 13). The app is the primary product; the CLI is scriptable standalone.

> **App-first rule:** every GUI action maps 1:1 to a CLI subcommand because both
> hit the same Rust core functions. Logic lives in the Rust core, not in the
> webview.

---

## 🧑‍💼 Agent Catalog

| Family | Primary | Sub-agents | Scope |
| --- | --- | --- | --- |
| `architect` | architect.md | systems-designer, module-contractor | System design, module topology, ADRs, contract locking |
| `scraper` | scraper.md | archive-scraper, game-page-scraper, fixture-engineer | lewdzone.com page scraping, fixtures, HTML parsing |
| `resolver` | resolver.md | token-prober, dispatch-builder | Go-link token resolution via start→reveal API |
| `database` | database.md | schema-designer, sync-orchestrator | SQLite schema, migrations, sync pipeline |
| `dm` | dm.md | folder-organizer | Direct-stream routing, OS-native dispatch, folder folding |
| `cli` | cli.md | command-designer, output-formatter | Native Rust CLI, subcommands, GUI/CLI parity |
| `gui` | gui.md | app-shell, view-designer | Tauri 2 desktop app, Rust core, Svelte views, shared-core commands |
| `shortcuts` | shortcuts.md | artwork-fetch, shortcut-builder | SteamGridDB artwork, per-OS native shortcuts |
| `content` | content.md | provider-registry, steamgriddb-provider, vndb-provider, igdb-provider, itch-provider, steam-provider, indiedb-provider | External info + art enrichment (VNDB, IGDB, Steam, itch.io, IndieDB) |
| `testing` | testing.md | fixture-crafter, mock-engineer, test-suite-architect, debugger | Test layers, fakes, coverage floors |
| `review` | review.md | security-auditor, perf-auditor | Gate pipeline, threat modeling, perf budgets |

See [wiki/Agents](wiki/Agents) for the full agent catalog with sub-agent details.

---

## 📏 Rules

| # | Name | Scope |
| --- | --- | --- |
| 00 | Governance & Delegation | Command chain, one-owner-per-artifact |
| 01 | Code Style (Rust) | cargo fmt, cargo clippy -D warnings, rustfmt defaults |
| 02 | Naming Conventions | Canonical vocab, file naming, enums |
| 03 | Module Architecture | src-tauri/src crate layout, module boundaries, contracts |
| 04 | Remote Issue Protocol | Roadmap-first, sub-issue lifecycle, commit format |
| 05 | Network Etiquette | 1 req/s, retry, offline fixtures, live opt-in |
| 06 | SQLite Conventions | WAL, FK, migrations, store tokens not URLs |
| 07 | Download Dispatch | Direct-stream routing, OS-native dispatch, folder folding |
| 08 | Release Standards | SemVer, version sync, verification gate |
| 09 | Mermaid Standards | GitHub v10, quoted labels, ≤12 nodes |
| 10 | Security | Secrets, allowlists, subprocess safety |
| 11 | Testing | Rust unit/integration layers + Svelte Vitest, coverage floor |
| 12 | Error Handling | Exit codes 0–5, typed errors, stdout vs stderr |
| 13 | GUI Conventions | Shared-core invoke model, GUI/CLI parity guardrails |

Full descriptions: `.agents/rules/index.md` and `wiki/Design-Conventions`.

---

## ✅ Verification

Before any merge to `main`, all of the following must pass:

```bash
# Rust core: formatting, linting, types, tests (from src-tauri/)
cargo fmt --check
cargo clippy -- -D warnings
cargo check
cargo test

# Svelte frontend: types + unit tests (from repo root)
npm run check          # svelte-check
npm run test           # Vitest unit tests for frontend views

# GUI build
npm run tauri build    # all platforms

# CLI smoke
lewdzone --version
lewdzone --help
```

---

## 🏛️ Governance Principles

1. **One owner per artifact.** A file or module has exactly one owning agent.
2. **One core, two entry points.** The GUI and the CLI call the same Rust core
   functions. The app is the primary product; the CLI is scriptable standalone.
3. **Rules before code.** No implementation starts without its governing rule.
4. **Fail loudly.** Never swallow errors to keep the build green.
5. **ADR before contract change.** Any cross-layer interface change needs an ADR
   filed in `.agents/adr/` before implementation begins.

See [Rule 00: Governance & Delegation](.agents/rules/rule-00-governance.md)
for the full command chain and delegation matrix.