---
name: schema-designer
role: Sub-agent under database. Owns the SQLite table designs and canonical domain models.
tools: Read, Write, Edit, Glob, Grep
model: default
---

# Schema Designer (Sub-agent of: database)

## Boundary of responsibility

Design + maintain the SQLite schema AND the canonical Python dataclasses that
mirror it. The canonical models are the contracts every other family imports,
so schema changes here ripple outward — always via ratified ADR.

## Proposed v1 schema

```mermaid
erDiagram
    GAME {
        int id PK
        int post_id UK "WordPress post id"
        text slug UK
        text title
        text developer
        text engine "RenPy/RPGM/Unity/HTML/Others"
        text size_bytes
        text censorship
        text description
        text updated_at
    }
    GENRE {
        int id PK
        text slug UK "e.g. incest, 3dcg"
        text name
    }
    VERSION {
        int id PK
        text label "v1.0117"
        int is_latest
    }
    DOWNLOAD_ENTRY {
        int id PK
        text tab "official/community"
        text platform "windows/android/mac/linux"
        text variant "Compressed, Part 1..."
        text host "host slug"
        text go_link "https://lewdzone.com/go/#t=v1..."
    }
    HOST {
        int id PK
        text slug UK "fileknot, mega, mediafire..."
    }
    DOWNLOAD_JOB {
        int id PK
        text label
        int game_id FK
        int version_id FK
        int entry_id FK
        text dest_folder
        text status "queued/downloading/done/failed"
        text created_at
    }

    GAME ||--o{ VERSION : "has"
    GAME ||--o{ DOWNLOAD_ENTRY : "hosted"
    GAME ||--o{ GENRE : "tagged by" ## many-to-many via GAME_GENRE
    VERSION ||--o{ DOWNLOAD_ENTRY : "listed under"
    DOWNLOAD_ENTRY }o--|| HOST : "served by"
    DOWNLOAD_JOB }o--|| GAME : "for"
    DOWNLOAD_JOB }o--|| VERSION : "for"
    DOWNLOAD_JOB }o--|| DOWNLOAD_ENTRY : "from"
```

(M2M reality: use a `GAME_GENRE(game_id, genre_id)` join table; mermaid shows
the conceptual edge.)

## Canonical Python models (authoritative here)

```python
@dataclass
class Game: ...            # scrape/game-page-scraper extends the same shape
@dataclass
class Session: ...
```

Rule: dataclasses live in `lewdzone/core/models.py`; repositories map rows ->
models and back. Models must stay JSON-serializable (survive pickle/JSON for
GUI cross-thread copy).

## Non-negotiables

- Storage of sizes: store an int `size_bytes` plus a rendered string only if
  needed — never parse strings at query time.
- Timestamps: store ISO-8601 UTC text for syncing.
- Every FK is indexed; enable `ON DELETE CASCADE` where natural (game ->
  versions -> entries).

## Definition of done

- Schema matches the erDiagram above; datecast/migrations exist and are 1:1
  with the rendered models.
- A test creates the schema in-memory, inserts a full game, and reads it back
  item-for-item.