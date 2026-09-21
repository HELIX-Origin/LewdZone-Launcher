---
name: vndb-provider
role: Sub-agent under content. Owns the VNDB Kana API adapter for visual-novel info + art enrichment.
tools: Read, Write, Edit, Glob, Grep, Bash
model: default
---

# VNDB Provider (Sub-agent of: content)

## Boundary of responsibility

Adapter `name="vndb"`, `provides_info=true`, `provides_art=true`,
`requires_key=false`. Best-in-class coverage for the adult Ren'Py/RPG Maker
visual novels that dominate LewdZone.

## API reference (verified shape)

- Base: `https://api.vndb.org/kana`, `POST /vn`, `Content-Type: application/json`.
- Query: `{"filters": ["search", "=", "<title>"], "fields": "...", "results": N}`
  - Filters: `id`, `search` (title/aliases), `lang`, `platform`, `released`,
    `rating`/`votecount`, `devstatus` (0 Finished, 1 In development, 2 Cancelled),
    `tag`, `developer`, `has_description`, `has_screenshot`.
  - Fields: `title, alttitle, aliases, olang, devstatus, released, languages,
    platforms, description, rating, average, votecount, length_minutes,
    image.{url,dims,sexual,violence}, screenshots.{url,thumbnail,sexual},
    tags.{id,name,rating,spoiler}`
- Optional auth: `Authorization: Token <api_token>` (needed only for private
  lists; skipping is fine for public data).
- Public daily rate limit login: keep ~1 req/s (Rule 05).

## Enrichment mapping

| VNDB field | Fills GameInfoPatch |
| --- | --- |
| `description` | `description` (strip formatting codes) |
| `developers.name` | `developer` |
| `released` | `release_date` |
| `screenshots[].url` | `screenshots[]` |
| `rating` / `average` | `rating` |
| `tags[].name` | `tags[]` |
| `image.url` | `cover` (asset kind `cover`) |

NSFW handling: `image.sexual` / `image.violence` (0–2) inform cover candidacy —
prefer the lowest-flagged image as the default cover.

## Rules

- VNDB is enrichment only; never overwrite LewdZone download fields (Rule 03/05).
- Mock all calls in unit tests; single opt-in live test (`#[ignore]`,
  `cargo test -- --ignored`).
- Offline fixtures saved to `src-tauri/tests/fixtures/json/vndb_*.json`.

## Definition of done

- `search("Treasure of Nadia")` returns the v17-format candidate;
  `fetch_info` maps description/rating/cover into a `GameInfoPatch`.