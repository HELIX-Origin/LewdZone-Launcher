---
name: adr
scope: architecture decision records for cross-layer contract changes
---

# Architecture Decision Record

> **When to file:** any cross-layer interface change (per [Rule 03](../rules/rule-03-module-architecture.md)) — a new contract, a modified signature, a new external seam, or a re-layering. Rule 04 Sub-Issue 1 (diagnostics & architecture) is where ADRs land.
>
> **Where:** `.agents/adr/NNNN-title.md`. **Status flow:** `proposed → accepted → superseded-by-XXXX`.

# ADR-XXXX: <Short Decision Title>

- **Status:** proposed | accepted | superseded
- **Date:** YYYY-MM-DD
- **Owner:** <owning agent family>
- **Applies to rule(s):** Rule 03 (+ related)

## Context

What problem prompted this decision? What are the constraints (Rule 03 layers,
Rule 06 data, Rule 07 download dispatch, Rule 10 security, Rule 12 errors)? Include
the actors: the CLI engine, the Tauri app shell, controllers, services, and
external seams involved.

## Decision

State the contract precisely. Include signatures, message shapes (JSON,
table/column changes, or process boundaries. Be specific enough that an
implementer does not need to re-decide.

## Consequences

- **Benefits:** …
- **Costs / risks:** …
- **Migration:** forward-only steps; how existing data/behavior is preserved.

## Alternatives considered

1. <option A> — rejected because …
2. <option B> — rejected because …

## Verification

- [ ] Contract documented at its owner (agent family doc)
- [ ] Tests assert the shape (fakes in `src-tauri/tests/support/`, parity tests)
- [ ] Wiki page updated (relative links, no `.md` extension)
- [ ] ROADMAP/TODO updated if scope changed