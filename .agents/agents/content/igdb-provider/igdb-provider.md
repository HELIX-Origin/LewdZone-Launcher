---
name: igdb-provider
role: Sub-agent under content. Owns the IGDB v4 adapter for metadata + cover/artwork enrichment.
tools: Read, Write, Edit, Glob, Grep, Bash
model: default
---

# IGDB Provider (Sub-agent of: content)

## Boundary of responsibility

Adapter `name="igdb"`, `provides_info=True`, `provides_art=True`,
`requires_key=True` (Twitch Client-ID + access token exchange).

## Auth flow

1. User supplies `igdb_client_id` + `igdb_client_secret` in settings.
2. POST `https://id.twitch.tv/oauth2/token` with `client_id`,
   `client_secret`, `grant_type=client_credentials`, `scope=` ->
   `{access_token, expires_in}`.
3. Cache the token until near expiry; refresh lazily.

## API reference (verified shape)

- Base: `https://api.igdb.com/v4`, POST endpoints, headers:
  `Client-ID: <id>`, `Authorization: Bearer <token>`.
- Search: `POST /games` with `search "<title>"; fields name, first_release_date,
  cover.image_id, screenshots.image_id, genres.name, summary; limit N;`
- Details: `POST /games` by id with extended fields.
- Artwork: `POST /artworks`, covers: `POST /covers` (`image_id`).
- Image URL pattern: `https://images.igdb.com/igdb/image/upload/t_{size}/{image_id}.jpg`
  (sizes: `cover_big`, `720p`, `1080p`, `thumb`).

## Enrichment mapping

| IGDB field | Fills GameInfoPatch |
| --- | --- |
| `summary` | `description` |
| `first_release_date` | `release_date` (epoch→iso) |
| `genres[].name` | `tags[]` |
| `cover` / `artworks` | `cover` / artwork asset kinds |
| `screenshots[].image_id` | `screenshots[]` |

## Rules

- Adult-title coverage is limited vs VNDB; dispatch order keeps VNDB first for
  VN-genre titles, IGDB for larger catalog matches.
- Token never logged; refreshed via client_credentials (no user token).
- Mock all calls; `-m live` opt-in test; fixtures `tests/fixtures/json/igdb_*.json`.

## Definition of done

- `fetch_info` + `fetch_cover` for a mainstream title with real key in live
  tests; offline suite uses fixtures.