---
name: commit-message
description: Conventional commit message format for the repository
---

# Commit Message Format

Follow [Conventional Commits](https://www.conventionalcommits.org/) with the
project's type scopes.

```
<type>(<scope>): <short summary in imperative mood>

<body: what changed and why. Keep lines ≤ 72 chars.>

Refs: #<issue>
```

## Types

- `feat` — new user-facing feature
- `fix` — bug fix
- `docs` — documentation or `.agents`/wiki changes
- `style` — formatting only (no logic change)
- `refactor` — code change that neither fixes a bug nor adds a feature
- `test` — adding or correcting tests
- `chore` — build, deps, tooling, CI
- `ci` — CI/CD changes
- `revert` — revert a previous commit

## Scopes

- `core` — Rust core logic
- `gui` — Svelte frontend
- `content` — content-provider layer
- `download` — download dispatch / queue
- `library` — library/launch
- `shortcuts` — native shortcuts
- `ci` — workflows
- `release` — version bumps, changelogs
- `agents` — `.agents/` docs

## Examples

```
feat(download): add queue persistence for download jobs

core::queue now writes every state change to the queue_job table so
the Downloads view survives app restarts.

Refs: #1
```

```
docs(agents): rewrite rule-07 for in-app streaming dispatch
```
