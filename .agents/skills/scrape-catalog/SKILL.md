---
name: scrape-catalog
description: Fetch and parse the lewdzone.com /games/ archive listings (with platform/engine/sort/state filters and pagination) and genre pages into GameCard lists. Use when populating or refreshing the catalog.
---

# Scrape Catalog

Collect game cards from the archive (`/games/`) and genre listings
(`/game-genre/<slug>/`), handling pagination and filters.

## Workflow

```mermaid
flowchart TD
    A[options: platform/engine/sort/state] --> B[build archive URL]
    B --> C[GET /games/ page 1]
    C --> D[archive-parser -> GameCard[] + meta]
    D --> E{more pages?}
    E -- yes --> F[fetch next page (or /page/N/)] --> C
    E -- no --> G[aggregate cards]
    G --> H[return catalog + meta]

    style C fill:#2f6f4f,color:#fff
    style D fill:#4b6e91,color:#fff
    style G fill:#874b4b,color:#fff
```

## Steps

1. Build the archive URL:
   - Base `https://lewdzone.com/games/`
   - Query params: `platform=PC|Android|Linux|Mac`,
     `engine=RenPy|RPG+Maker|Unity|...`, `sort=Popularity|New+to+Old|...`,
     `state=Ongoing|...`.
2. Fetch + parse page 1 -> `GameCard[]` + `ArchiveMeta`.
3. Pagination: **determine empirically** — check whether `/games/?page=2` or
   `/games/page/2/` works; record the finding in `docs/parsing/archive.md`.
4. Walk remaining pages with polite delays; normalize platform/engine to
   canonical slugs.
5. Return the aggregate list; caller decides db upsert (see `sync-catalog-db`).

## Rules

- Never follow into individual game pages here (`game-page-scraper` does that).
- Resolve relative hrefs to absolute; strip tracking.
- Filter choices must round-trip: applying the same options must reproduce the
  same card set (tested against the saved archive fixture).

## Checkoffs

- [ ] Pagination scheme verified and documented
- [ ] Fixture `lz_archive.html` parse test green
- [ ] Platform/engine/sort normalization unit tests