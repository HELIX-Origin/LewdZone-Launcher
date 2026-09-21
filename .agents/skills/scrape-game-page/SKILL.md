---
name: scrape-game-page
description: Fetch and parse a single lewdzone.com /game/<slug>/ page into the canonical Game model with versions and download entries. Use when you need one game's full metadata, download rows, or go-links from the site.
---

# Scrape Game Page

Fetch + parse one game detail page, offline-testable against the fixture.

## Workflow

```mermaid
flowchart LR
    A[slug or URL] --> B[scraping/fetch.py GET]
    B --> C[game-page-parser]
    C --> D[Game model]
    C --> E[versions -> DownloadEntry lists]
    D --> F[db upsert (optional)]
    E --> R[hand go-links to resolver]

    style C fill:#2f6f4f,color:#fff
    style D fill:#4b6e91,color:#fff
```

## Steps

1. **Input**: slug (`treasure-of-nadia`) or full URL
   `https://lewdzone.com/game/<slug>/`.
2. **Fetch** via `scraping/fetch.py` — headers UA `Mozilla/5.0`, polite delay
   (see `network-etiquette`), return HTML.
3. **Parse** with `game-page-parser.parse(html, base_url) -> Game`:
   - title, developer, current_version, engine, platforms, size, censorship,
     genres (from `/game-genre/<slug>/` tag links), screenshots, description.
   - versions from the version dropdown; `(Latest)` marker.
   - for each version: official + community tabs -> `DownloadEntry` rows
     (label, variant, host, go_link). Do NOT decode go-links here.
4. **Verify** against fixture: `lz_game.html` must yield post_id 18212,
   developer "NLT Media", 47 download entries.
5. **Output**: pass `Game` to the caller (db/sync or CLI `info`).

## Rules

- Pure parser: no network inside the parser itself.
- Absent sections -> `None`/`[]`; never crash.
- Size via shared `parse_size()` helper (`7.51 GB` -> bytes).

## Checkoffs

- [ ] Fixture test green (`tests/unit/scraping/test_game_page.py`)
- [ ] No network call inside parser
- [ ] Field map in `docs/parsing/game-page.md` updated if changed