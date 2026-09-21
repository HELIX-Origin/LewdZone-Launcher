---
name: rules
description: The lewdzone-launcher rule library - every rule that governs how agents, CLI, GUI, data, and releases are built and maintained.
---

# Rules

The lewdzone-launcher rule library. Rules are the **contract** agents must follow.
They are numbered `00`–`13`, collected in [`index.md`](./index.md), and every
rule is enforceable by the review gate (see
[agents/review](../agents/review/review.md)).

## Purpose

Rules exist so that every agent — architect, scraper, resolver, database, dm,
cli, gui, shortcuts, testing, review — agrees on invariants before touching
code. A rule violation is a review failure, not a style nit.

## Reading order

Read [`index.md`](./index.md) first. It maps each rule to the agents that own
it and the artifacts it constrains. Then read the rules for the layer you are
working in:

| Layer | Rules |
| --- | --- |
| Governance | `00` |
| Frontends | `01`, `02`, `03`, `12`, `13` |
| Domain + storage | `03`, `06` |
| Network + resolution | `05`, `07`, `10` |
| Delivery to GitHub | `04`, `08`, `09` |
| Quality gate | `11` |

## Authoring rules

New rules follow the [`rule` template](../templates/rule.md). Every rule:

1. Has YAML frontmatter (`name`, `rule_number`, `scope`, `enforcement`).
2. States edge cases and failure modes explicitly.
3. Contains **one** GitHub-compliant mermaid diagram per
   [Rule 09](./rule-09-mermaid-standards.md).
4. Lists acceptance criteria a reviewer can check.

## Amending rules

Rules are normative. Changing one requires an
[ADR](../templates/adr.md) and a review-gate sign-off — never a silent edit.