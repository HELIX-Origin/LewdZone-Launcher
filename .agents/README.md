# 🤖 Agent Ecosystem

This directory contains the agent-facing governance layer for the
LewdZone Launcher project. It is plain Markdown with YAML frontmatter,
designed to be consumed by coding agents (opencode, GitHub Copilot-style
workflows, and others).

## Layout

```
.agents/
  agents/{family}/{family}.md          # primary agent per family
  agents/{family}/{sub-agent}/{sub}.md # sub-agents
  skills/{name}/SKILL.md               # reusable procedures
  rules/rule-[00..13]-*.md + index.md  # enforceable conventions
  templates/                           # reusable scaffolds
  adr/                                 # architecture decision records
  README.md                            # this file
```

## Families

| Family | Scope |
| --- | --- |
| `architect` | System design, module topology, ADRs, contract locking |
| `scraper` | lewdzone.com page scraping, fixtures, HTML parsing |
| `resolver` | Go-link token resolution via start→reveal API |
| `database` | SQLite schema, migrations, sync pipeline |
| `dm` | **Downloads + folder folding** — direct-stream routing, OS-native dispatch, folder-organizer |
| `cli` | Native Rust CLI, subcommands, GUI/CLI parity |
| `gui` | Tauri 2 desktop app, Rust core, Svelte views |
| `shortcuts` | SteamGridDB artwork, per-OS native shortcuts |
| `content` | External info + art enrichment (VNDB, IGDB, Steam, itch.io, IndieDB) |
| `testing` | Test layers, fakes, coverage floors |
| `review` | Gate pipeline, threat modeling, perf budgets |

## Key model changes

- **Download dispatch** (Rule 07) no longer uses external download managers.
  Direct-file hosts stream in-app; other hosts open via the OS default handler.
  See `agents/dm/dm.md` and `adr/0002-download-manager-adapters.md`.
- **API keys** live in the SQLite `secret` table, not the JSON config.
- **Install layout**: `downloads/` for staging, `lzapps/<slug>/` for extracted
  games, plus an itch.io-style `app.json` manifest.

## Entry points

- Rules index: [`rules/index.md`](rules/index.md)
- Agent catalog: [`agents/architect/architect.md`](agents/architect/architect.md)
- ADRs: [`adr/`](adr/)
- Verification gates: [`rules/rule-01-code-style-rust.md`](rules/rule-01-code-style-rust.md)
