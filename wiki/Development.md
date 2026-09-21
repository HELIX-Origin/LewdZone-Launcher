# Development

> Links between wiki pages are relative and omit the `.md` extension.

This project is governed by a **detailed agent ecosystem** in `.agents/` with
GitHub-compatible Mermaid diagrams, plus rules 00-13. The wiki is the
human-facing distillation; the `.agents` docs are the authoritative spec.

## Repo layout

```
lewdzone-launcher/
  .agents/
    agents/                # 10 agent families + sub-agents
    skills/                # SKILL.md per skill
    rules/                 # rule-00..13 + index
    templates/             # agent/skill/rule/module/adr templates
  src/lewdzone_launcher/   # Python CLI engine (src layout)
  desktop/                 # Tauri 2 app (src-tauri/ Rust, src/ Svelte)
  tests/                   # pytest suite
  wiki/                    # this wiki (synced with GitHub Wiki)
  scratch/                 # gitignored temp scripts
```

## Agent families

| Family | Owns |
| --- | --- |
| architect | structure, ADRs, contracts (systems-designer, module-contractor) |
| scraper | site scraping, fixtures (archive-scraper, game-page-scraper, fixture-engineer) |
| resolver | go-token → real URL (token-prober, dispatch-builder) |
| database | schema, sync (schema-designer, sync-orchestrator) |
| dm | download managers (dm-detector, fdm-adapter, idm-adapter, torrent-adapter, folder-organizer) |
| cli | the engine, output (command-designer, output-formatter) |
| gui | Tauri app (app-shell, view-designer, sidecar-driver) |
| shortcuts | artwork + native shortcuts (artwork-fetch, shortcut-builder) |
| testing | suite, fakes (fixture-crafter, mock-engineer, test-suite-architect, debugger) |
| review | gates (security-auditor, perf-auditor) |

See [Agent Ecosystem](Agents).

## Rules (00-13)

Governance, code style, naming, module architecture, remote issue protocol
(roadmap-first), network etiquette, SQLite conventions, download-manager
integration, release standards, Mermaid standards, security, testing, error
handling, GUI conventions. Index: [`.agents/rules/index.md`](../.agents/rules/index.md) (wiki link; repo path has `.md`).

## Contribution workflow

1. Issues are **roadmap-first**: a plan has one living roadmap issue, edited in
   place; work is tracked as sub-issues.
2. Commits follow `<emoji> <type>(<scope>): <subject>` with `Resolves #N` /
   `Closes #N`. Scopes include `scraper`, `resolver`, `db`, `dm`, `cli`, `gui`,
   `shortcuts`, `tests`, `agents`, `rules`, `docs`, `deps`.
3. Every plan/bug issue embeds ≥1 GitHub-compatible Mermaid diagram.
4. PRs mirror issues (`Part of #parent` / `Closes #sub-issue`); bodies via
   `gh pr create --body-file`.

Full protocol: [Rule 04](../.agents/rules/rule-04-remote-issue-protocol).

## Branch & commit hygiene

- Feature branch → PR → merge to `main`. Never push directly.
- Clean tree before commits; never commit secrets
  ([Security](Security)).
- Changes to cross-layer contracts require an ADR
  ([Design Conventions](Design-Conventions)).

## Verification commands

```sh
ruff check . && ruff format --check .
pyright
pytest -q                      # offline default
pytest -m live                 # opt-in live
pytest --cov --cov-fail-under=85
```

See [Testing & QA](Testing) and [Release Process](Release-Process).