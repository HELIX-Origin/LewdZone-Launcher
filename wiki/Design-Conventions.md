# Design Conventions

> Links between wiki pages are relative and omit the `.md` extension.

## Naming

| Item | Convention |
| --- | --- |
| Python files / vars / table names | `snake_case` |
| Python types / classes / agents | `PascalCase` |
| CLI commands / subcommands | `kebab-case` |
| Domain enum values | UPPER_CASE (`Platform.PC`, `Platform.ANDROID`) |
| Tauri / JS variables | `camelCase` |
| Download files | `<Title> - <Version> - <Platform>[- <Variant>].<ext>` |
| Shortcut group / folder | `lewdzone` |

## Canonical vocabulary

`Game` / `PostId`, `Version`, `DownloadEntry`, `Host` (slug from the site),
`GoToken` (`v1.<payload>.<sig>`), `DownloadJob`, `ArtworkCache`, `Shortcut`,
`Genre`.

## Code style

- **Ruff** format + lint; **Pyright** strict
- Black-compatible (88-char), double quotes
- `isort` via ruff; full type annotations; no bare `Any`
- `NewType` / `TypeAlias` for domain IDs (`PostId`, `GameId`)
- `Literal` and discriminated unions over flag booleans

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
before implementation, filed in `docs/adr/` using the template
[`templates/adr.md`](../.agents/templates/adr.md).