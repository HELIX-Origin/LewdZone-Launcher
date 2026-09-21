---
name: steamgriddb-provider
role: Sub-agent under content. Owns SteamGridDB icon/grid/hero/logo fetching for the content layer.
tools: Read, Write, Edit, Glob, Grep, Bash
model: default
---

# SteamGridDB Provider (Sub-agent of: content)

## Boundary of responsibility

Adapter `name="steamgriddb"`, `provides_info=False`, `provides_art=True`,
`requires_key=True`. Supplies icons (primary) plus grids/heroes/logos for
covers and store-style display.

## API reference (verified shape)

- Base: `https://www.steamgriddb.com/api/v2`, auth `Authorization: Bearer <key>`
- Search: `GET /api/v2/search/autocomplete/{q}` -> `{success, data:[{id,name,types}]}`
- Icons: `GET /api/v2/icons/game/{game_id}`
- Grids: `GET /api/v2/grids/game/{game_id}?dimensions=600x900`
- Heroes: `GET /api/v2/heroes/game/{game_id}`
- Logos: `GET /api/v2/logos/game/{game_id}`
- Use `?nsfw=yes` so mature titles still return art; cache the choice.
- No data / 404 -> `None` (caller falls back to next provider/generic icon).

## Icon-to-.ico conversion

- Pillow writes 256x256 `.ico` containing 16/32/48/256 sizes (dependency under
  a `[artwork]` extra in pyproject.toml).
- Icons only for shortcut icons; grids/heroes/logos are for app display and
  Steam-library style presentation, never `.lnk` icons (aspect mismatch).

## Rules

- Cache keyed by `(normalized_title, kind)` in `artwork_cache` with
  `provider="steamgriddb"` so rebuilds are offline-fast.
- Respect network etiquette (Rule 05): polite quota, retries ≤3, offline
  fixtures for unit tests.
- Never log the API key or Authorization header.

## Definition of done

- `fetch(kind, title) -> Path | None` returns icon + cover for "Treasure of
  Nadia"; `None` (no crash) on unknown title; all calls mocked in unit tests,
  one opt-in live test with user key.