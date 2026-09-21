# Agent Ecosystem

> Links between wiki pages are relative and omit the `.md` extension.

The repository is governed by an **extensible agent ecosystem** living in
`.agents/` — plain Markdown with standardized YAML frontmatter, designed to be
consumable by any agent that reads Markdown (works with opencode, GitHub
Copilot-style workflows, and others). Everything is documented with
**GitHub-compatible Mermaid diagrams**.

## Layout

```
.agents/
  agents/{family}/{family}.md          # primary agent per family
  agents/{family}/{sub-agent}/{sub}.md # sub-agents
  skills/{name}/SKILL.md               # reusable procedures
  rules/rule-[00..13]-*.md + index.md  # enforceable conventions
  templates/                           # reusable scaffolds
  README.md                            # the ecosystem index
```

## Families at a glance

| Family | Primary | Sub-agents |
| --- | --- | --- |
| **architect** | structure + contracts | systems-designer, module-contractor |
| **scraper** | catalog + page scraping | archive-scraper, game-page-scraper, fixture-engineer |
| **resolver** | go-token resolution | token-prober, dispatch-builder |
| **database** | schema + sync | schema-designer, sync-orchestrator |
| **dm** | download managers | dm-detector, fdm-adapter, idm-adapter, torrent-adapter, folder-organizer |
| **cli** | the engine | command-designer, output-formatter |
| **gui** | Tauri app | app-shell, view-designer, sidecar-driver |
| **shortcuts** | artwork + shortcuts | artwork-fetch, shortcut-builder |
| **testing** | QA suite | fixture-crafter, mock-engineer, test-suite-architect, debugger |
| **review** | gates | security-auditor, perf-auditor |

## Skills

Reusable procedures with checkoffs, e.g.:

- `scrape-game-page` — parse a game detail page into canonical models
- `scrape-catalog` — page the archive into the catalog
- `resolve-go-token` — the two-step start/reveal resolution
- `parse-version-prompts` — normalize version strings
- `launch-fdm` — resolve + dispatch to a manager (DM-agnostic)

## Rules 00-13

| # | Rule | File |
| --- | --- | --- |
| 00 | Governance | `rule-00-governance.md` |
| 01 | Code style (python) | `rule-01-code-style-python.md` |
| 02 | Naming conventions | `rule-02-naming-conventions.md` |
| 03 | Module architecture | `rule-03-module-architecture.md` |
| 04 | Remote issue protocol | `rule-04-remote-issue-protocol.md` |
| 05 | Network etiquette | `rule-05-network-etiquette.md` |
| 06 | SQLite conventions | `rule-06-sqlite-conventions.md` |
| 07 | Download manager integration | `rule-07-download-manager-integration.md` |
| 08 | Release standards | `rule-08-release-standards.md` |
| 09 | Mermaid standards | `rule-09-mermaid-standards.md` |
| 10 | Security | `rule-10-security.md` |
| 11 | Testing | `rule-11-testing.md` |
| 12 | Error handling | `rule-12-error-handling.md` |
| 13 | GUI conventions | `rule-13-gui-conventions.md` |

Index: [`.agents/rules/index.md`](../.agents/rules/index.md) (wiki link; repo
path has `.md`).

## Templates

Scaffolds for: agents, skills, rules, ADRs, Python modules, tests, migrations,
issues, roadmaps, release notes, and commit messages — all under
`.agents/templates/`.

## Governance principles

1. **One owner per artifact.** Two agents on one module is an anti-pattern.
2. **CLI is primary.** App and GUI are built on the same engine.
3. **Rules before code.** Unwritten rule + code = unreviewable.
4. **Fail loudly.** Errors surface with codes, not swallowed.
5. **ADR before contract change.**

Full detail: [Rule 00](../.agents/rules/rule-00-governance).