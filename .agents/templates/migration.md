---
name: migration
description: Template for a new SQLite schema migration
---

# Migration {{ number }}: {{ Title }}

## Purpose

{{ Why this schema change is needed. }}

## SQL

```sql
{{ Forward-only SQL. No ALTER DROP of user data without explicit user decision. }}
```

## Rollback

{{ Not applied (Rule 06 forward-only). Describe how to recover if needed. }}

## Data migration

{{ If existing rows must be transformed, describe the plan. }}

## Verification

- [ ] Migration is idempotent (`CREATE TABLE IF NOT EXISTS`, `CREATE INDEX IF NOT EXISTS`).
- [ ] Added to `db::migrations::MIGRATIONS` in lexical order.
- [ ] Table/index name added to `db::tests::core_tables_exist_after_migrate`.
- [ ] Repo helpers and tests added/updated.
- [ ] `cargo test` passes.
