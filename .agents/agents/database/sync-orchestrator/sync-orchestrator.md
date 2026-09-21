---
name: sync-orchestrator
role: Sub-agent under database. Owns pulling the site catalog into SQLite and keeping it fresh.
tools: Read, Write, Edit, Glob, Grep, Bash
model: default
---

# Sync Orchestrator (Sub-agent of: database)

## Boundary of responsibility

Coordinate scraper parsers + fetch helper to fill the DB, and keep it fresh:

- Full sync (all archive pages, all games).
- Targeted sync by filter/genre.
- Incremental update check (has this game changed since `updated_at`?).
- Prune deleted games / entries when the site removes them.

## Sync pipeline

```mermaid
flowchart LR
    A[full or filter sync] --> B[fetch archive page 1]
    B --> C{more pages?}
    C -- yes --> D[fetch next page]
    D --> B
    C -- no --> E[collect GameCards]
    E --> F["group: new vs changed vs missing"]
    F --> G["for each new/changed: fetch game page"]
    G --> H[parse Game + entries]
    H --> I[upsert into DB in one transaction per page]
    I --> J[prune rows not seen on site]
    J --> K["synced catalog"]

    style F fill:#4b6e91,color:#fff
    style I fill:#2f6f4f,color:#fff
    style K fill:#874b4b,color:#fff
```

## Sync rules

1. **Batching**: commit per-page, not per-game; wrap page parse + upsert in one
   transaction.
2. **Politeness**: min delay between fetches; resumable (record last successful
   page/offset in a `sync_state` table so a crash continues, not restarts).
3. **Idempotent upserts**: same slug/post_id -> update, never duplicate.
4. **Pruning policy**: only prune when a full sync completes without errors; a
   partial/failed sync must NOT delete rows (avoid mass wipe on transient site
   errors).
5. **Post IDs are the stable identity**; slug can change if the site renames a
   permalink.

## Incremental check flow

```mermaid
sequenceDiagram
    participant Sync as sync-orchestrator
    participant DB as SQLite
    participant S as lewdzone.com

    Sync->>DB: SELECT updated_at for game
    Sync->>S: GET /game/slug/ (cheap HEAD if possible)
    S-->>Sync: page meta
    Sync->>Sync: compare last-modified / meta hash
    alt changed
        Sync->>S: parse full page
        Sync->>DB: upsert game + entries
    end
```

## Definition of done

- Full sync of fixture-based catalog completes with resume-after-failure test.
- Prune never runs on a failed/partial sync (regression test covers this).
- Upsert is provably idempotent (run twice -> same row count).