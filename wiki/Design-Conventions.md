# Design Conventions

> Links between wiki pages are relative and omit the `.md` extension.

## Naming

| Item | Convention |
| --- | --- |
| Rust files / vars / DB table names | `snake_case` |
| Rust types / structs / enum variants | `PascalCase` (e.g. `Platform::PC`) |
| CLI commands / subcommands | `kebab-case` |
| Rust consts / serialized enum values | UPPER_CASE (`"PC"`, `"ANDROID"`) |
| Svelte / JS variables | `camelCase` |
| Download files | `<Title> - <Version> - <Platform>[- <Variant>].<ext>` |
| Shortcut group / folder | `lewdzone` |

## Canonical vocabulary

`Game` / `PostId`, `Version`, `DownloadEntry`, `Host` (slug from the site),
`GoToken` (`v1.<payload>.<sig>`), `DownloadJob`, `ArtworkCache`, `Shortcut`,
`Genre`.

## Code style

- **rustfmt** (`cargo fmt`) + **clippy** with `-D warnings` as the gate; `cargo check`
- Module-level `pub` types documented; no `unwrap()` outside tests and top-level
  entry points — use `?` with typed errors (Rule 12)
- Newtype wrappers for domain IDs (`PostId(u64)`, `GameId(u64)`, `GoToken`)
- Explicit `enum` variants over flag booleans

## Mermaid diagrams

All Mermaid in the repo must comply with GitHub's Mermaid v10.x renderer.
Rules:

- **Fence** with ` ```mermaid `.
- **Quote** every label containing special characters: `A["label (with parens)"]`,
  `A -->|"Yes"| B`.
- **Subgraph titles** must be quoted: `subgraph id["Title"]`.
- One concern per diagram; ≤8-12 nodes.
- Label both branches of every decision.
- No `%%{init}%%` directives; no math/LaTeX; no exotic shapes.
- No reserved-word IDs: `end`, `class`, `note`, `subgraph`, `link`,
  `default`, `linkStyle`.
- Allowed styling: only `classDef` / `linkStyle` / `style` color overrides.

Full rule: [Rule 09](../.agents/rules/rule-09-mermaid-standards).

## ADRs

Cross-layer contract changes require an **Architecture Decision Record**
before implementation, filed in `.agents/adr/` using the template
[`templates/adr.md`](../.agents/templates/adr.md).