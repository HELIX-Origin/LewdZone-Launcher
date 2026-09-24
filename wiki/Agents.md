# 🤖 Agent Ecosystem & Governance

> Links between wiki pages are relative and omit the `.md` extension.

The LewdZone Launcher repository is governed by a formalized **agent ecosystem** residing in `.agents/`. Each agent family has strict ownership boundaries, codified procedures (skills), and enforceable rules (Rule 00 through Rule 13).

---

## 📁 Ecosystem Layout

```text
.agents/
├── agents/{family}/{family}.md          # Primary agent specification per family
├── agents/{family}/{sub-agent}.md       # Sub-agent responsibilities
├── skills/{name}/SKILL.md               # Reusable engineering procedures
├── rules/rule-[00..13]-*.md             # Non-negotiable repository rules
├── templates/                           # Scaffolds for agents, rules, ADRs, and issues
└── adr/                                 # Architecture Decision Records
```

---

## 🧩 Agent Families & Ownership

| Family | Primary Responsibility | Key Sub-Agents |
| --- | --- | --- |
| **`architect`** | System topology, ADR governance, contract locking | `systems-designer`, `module-contractor` |
| **`scraper`** | HTML page parsing, catalog pagination, fixtures | `archive-scraper`, `game-page-scraper`, `fixture-engineer` |
| **`resolver`** | Go-link token resolution via start/reveal API | `token-prober`, `dispatch-builder` |
| **`database`** | SQLite migrations, schema design, sync pipeline | `schema-designer`, `sync-orchestrator` |
| **`dm`** | Direct-stream routing, 7-Zip CLI integration, folder folding | `folder-organizer` |
| **`cli`** | Native Rust CLI commands, JSON formatting | `command-designer`, `output-formatter` |
| **`gui`** | Tauri 2 application, Svelte 5 views, system tray | `app-shell`, `view-designer` |
| **`content`** | External metadata & artwork enrichment | `provider-registry`, `steamgriddb-provider`, `vndb-provider`, `igdb-provider` |
| **`testing`** | QA test layers, fakes, coverage floors | `fixture-crafter`, `mock-engineer`, `test-suite-architect`, `debugger` |
| **`review`** | Security threat modeling, performance budgets | `security-auditor`, `perf-auditor` |

---

## ⚖️ Repository Rules (00–13)

| # | Name | Governing Principle |
| --- | --- | --- |
| **00** | Governance & Delegation | Command chain, single owner per artifact. |
| **01** | Code Style | Rust formatting (`cargo fmt`), clippy (`-D warnings`). |
| **02** | Naming Conventions | Canonical vocabulary and identifier rules. |
| **03** | Module Architecture | Inward dependency rule, crate layer boundaries. |
| **04** | Remote Issue Protocol | Roadmap-first tracking, structured commit messages. |
| **05** | Network Etiquette | Rate-limited requests (1 req/s), bounded retries, offline testing. |
| **06** | SQLite Conventions | WAL mode, foreign keys ON, migrations, store tokens not URLs. |
| **07** | Download Dispatch | Direct-file streaming, OS-native dispatch, 7-Zip extraction. |
| **08** | Release Standards | Semantic versioning, verification gate before tag. |
| **09** | Mermaid Standards | GitHub v10 renderer compatibility, quoted node labels. |
| **10** | Security | Parameterized SQL, secret storage in DB, no shell interpolation. |
| **11** | Testing | Hermetic unit/integration tests, coverage floors. |
| **12** | Error Handling | Exit codes 0–5, strongly typed error models. |
| **13** | GUI Conventions | One core, two entry points; GUI/CLI parity. |

---

## 🏛️ Governance Principles

1. **One Owner per Artifact:** Every source file or subsystem has exactly one owning agent family.
2. **One Core, Two Entry Points:** The desktop application and the CLI must share identical core Rust logic (Rule 03, Rule 13).
3. **Rules Before Code:** Architectural rules precede implementation.
4. **Fail Loudly:** Never swallow errors or panic in core paths.
5. **ADR Before Contract Changes:** Breaking contract or layout changes require an ADR.

---

## 🔗 Related Pages

- [Architecture](Architecture)
- [Design Conventions](Design-Conventions)
- [Development Guide](Development)
- [Security](Security)