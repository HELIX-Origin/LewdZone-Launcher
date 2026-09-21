# ADR-0003: SQLite persistence (tokens, not URLs)

- **Status:** accepted
- **Date:** 2026-09-21
- **Owner:** database family
- **Applies to:** Rule 03, Rule 06, Rule 10

## Context

The catalog, jobs, artwork cache, and shortcuts need durable state. The go-link
tokens are time-limited and must be resolved fresh per download; resolved URLs
expire and are not safe to persist. Content-provider enrichment produces
external ids that must survive restarts. The tool runs on Win/Linux/macOS with
per-OS config/data dirs.

## Decision

- **SQLite** is the single source of persisted state, stored at
  `<data_dir>/lewdzone.db` (Rule 06): WAL mode, `foreign_keys=ON`,
  `busy_timeout=5000`, parameterized SQL only, single writer connection.
- **Store tokens, never resolved URLs.** `download_entry` rows hold the
  go-link token (`v1.payload.sig`); the resolver produces real URLs only at
  dispatch time (ADR-0002).
- **Forward-only migrations** `migrations/NNN_desc.py` applied in lexical
  order via a `schema_migrations` table; CI verifies a scratch DB migrates
  clean (Rule 06). No destructive migration ever rewrites history.
- Config is per-OS JSON at `config_dir()/config.json` (Rule 03 `_config.py`)
  with settings: `download-root`, `dm`, site flags, `content_*` keys.
  Secrets (`sgdb_api_key`, `igdb_client_id/secret`) validate as set/unset
  only — never echoed (Rule 10).

## Consequences

- **Benefits:** tokens are cheap to store and safe; migrations give forward
  provenance; categories (catalog/jobs/artwork/external ids) evolve
  independently without breaking old DBs.
- **Costs/risks:** WAL single-writer means serializing writes; stale catalog
  requires re-sync; token expiry means downloads need a fresh resolve.
- **Migration:** 001_initial creates the base schema; later tables via new
  numbered files only.

## Alternatives considered

1. Persist resolved URLs — rejected: they expire, leak host links, and
   bypass the resolver/allowlist (Rule 10).
2. JSON-file catalog — rejected: weak querying, no FK integrity, no
   forward migrations.

## Verification

- [ ] `001_initial.py` migration bootstraps an empty DB with `schema_migrations` row
- [ ] Unit tests: WAL on, FK enforced, busy_timeout set, parametrized upserts
- [ ] CI scratch-DB migration check green (Rule 06)
- [x] Wiki `Configuration` (`lewdzone.db`, WAL FK5000) documented