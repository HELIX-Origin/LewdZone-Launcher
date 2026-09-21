---
name: game-page-scraper
role: Sub-agent under scraper. Owns the full /game/<slug>/ detail parser.
tools: Read, Write, Edit, Glob, Grep
model: default
---

# Game Page Scraper (Sub-agent of: scraper)

## Boundary of responsibility

Parses a single `https://lewdzone.com/game/<slug>/` page into the full `Game`
model, including the download section with versions and go-links.

## Canonical models (bird's-eye; authority: database/schema-designer)

The canonical models are Rust structs owned by `database/schema-designer`
(`src-tauri/src/db/`):

- `Game { slug, post_id: i64` (the WP post id — stable identity), `title,
  developer: Option<String>, current_version: Option<String>,
  engine: Option<String>` (Ren'Py / RPGM / Unity / HTML / Others),
  `platforms: Vec<String>` (windows, android, mac, linux),
  `genres: Vec<String>` (tag slugs e.g. incest, 3dcg, harem),
  `size_bytes: Option<u64>` (parsed from e.g. "7.51 GB"),
  `censorship: Option<String>, screenshots: Vec<String>,
  description: Option<String>, versions: Vec<Version> }`
- `DownloadEntry { label, variant: Option<String>, host, go_link }` — `host`
  from the `d-<host>` class / go payload `h`; `go_link` the full
  `https://lewdzone.com/go/#t=v1...` href.
- `Version { label, is_latest, download_tabs: DownloadTabs }` where
  `DownloadTabs { official: Vec<DownloadEntry>, community: Vec<DownloadEntry> }`

## Parse map (game page -> models)

```mermaid
flowchart LR
    A[game page HTML] --> A1[meta section]
    A --> A2["action/info boxes"]
    A --> A3[genre tag links]
    A --> A4[download section]
    A1 --> B[title, developer, version]
    A2 --> C[size, censorship, engine, platforms, screenshots]
    A3 --> D[genres list]
    A4 --> E{version tabs}
    E --> F[official + community link rows]
    F --> G[DownloadEntry go-links]
    B --> H{Game model}
    C --> H
    D --> H
    G --> H

    style A fill:#2f6f4f,color:#fff
    style H fill:#874b4b,color:#fff
```

## Site-specific parse guidance (verified)

- Size strings like `7.51 GB`, `1.2 GB`, `850 MB` -> normalize via shared
  `parse_size()` helper.
- Download rows: platform label + optional `(Variant)` suffix; opposite hosts
  are paired in adjacent columns — distinguish Official vs Community by their
  parent tab, not by position.
- Version dropdown: `Choose Version` -> `vX.Y.Z (Latest)` marker for latest.
- Genres/tags: anchor hrefs to `/game-genre/<slug>/` — the slug is the tag id.
- `fixture-engineer` holds `lz_game.html` (treasure-of-nadia, 47 go-links) as
  the reference fixture; it must parse to 47 DownloadEntry objects.

## Rules

- 100% pure function: (html, base_url) -> Game. No requests.
- Every scraped attribute used downstream must be present in the fixture test.
- Never invent data; absent sections yield `None`/`[]`.

## Definition of done

- Fixture `lz_game.html` parses to Game with post_id 18212
  (`treasure-of-nadia`), title 'Treasure of Nadia', developer 'NLT Media',
  47 download entries, size 7.51 GB.
- Field map documented in this doc (`game-page-scraper.md`).