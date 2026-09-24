# 🎨 Design & Coding Conventions

> Links between wiki pages are relative and omit the `.md` extension.

This document records the coding, naming, diagramming, and architectural standards enforced across the LewdZone Launcher codebase.

---

## 🏷️ Naming Standards

| Scope | Convention | Examples |
| --- | --- | --- |
| Rust files / modules / DB tables | `snake_case` | `extract.rs`, `game_entry`, `download_job` |
| Rust structs / enums / traits | `PascalCase` | `ArchiveFilter`, `ResolvedUrl`, `AppJson` |
| CLI commands & flags | `kebab-case` | `download-root`, `source-priority`, `7z-path` |
| Serialization constants | `UPPER_CASE` | `"PC"`, `"ANDROID"`, `"WINDOWS"` |
| Svelte components | `PascalCase` | `GameCard.svelte`, `QueueItem.svelte` |
| Svelte & TS variables | `camelCase` | `activeDownloads`, `isExtracting` |
| Download archive names | Canonical pattern | `<Title> - <Version> - <Platform>[- <Variant>].<ext>` |

---

## 💬 Canonical Vocabulary

- **`Game` / `PostId`:** Canonical game entity and LewdZone numeric post identifier.
- **`Version`:** Specific release version string (`"0.19.1"`).
- **`DownloadEntry`:** Individual host option under a version/tab.
- **`GoToken`:** Ephemeral download token extracted from `#t=v1...` links.
- **`DownloadJob`:** Tracked queue task (`queued`, `resolving`, `downloading`, `extracting`, `complete`).
- **`7-Zip CLI`:** Standalone console binary (`7za`/`7z`/`7zz`) used for decompression.

---

## 📐 Rust & Frontend Code Style

- **Strict Formatting & Linting:** Code must pass `cargo fmt --check` and `cargo clippy -- -D warnings`.
- **Fail Loudly & Typed Errors:** No `unwrap()` in production core paths. Errors are represented by `core::Error` enums.
- **One Core, Two Entry Points:** Business logic resides strictly in `src-tauri/src/core/`. GUI invoke commands and CLI subcommands call identical core functions (Rule 03, Rule 13).
- **Inward Dependency Rule:** Outer presentation layers depend inward on services and models; inner layers never import presentation controllers.

---

## 🧩 Mermaid Diagram Rules

All diagrams in documentation and issues must comply with GitHub Mermaid v10.x standards:
- Always quote labels with special characters: `A["Label (with details)"]`.
- Quote subgraph titles: `subgraph "Extraction Pipeline"`.
- Keep diagrams focused and readable (≤ 8–12 nodes).
- Avoid reserved keywords as node identifiers (`end`, `class`, `note`, `subgraph`).

---

## 📜 Architectural Decisions (ADRs)

Any cross-layer interface change, storage model revision, or protocol addition requires an **Architecture Decision Record (ADR)** documented before implementation.

---

## 🔗 Related Pages

- [Architecture](Architecture)
- [Agent Ecosystem](Agents)
- [Testing & QA](Testing)
- [Security](Security)