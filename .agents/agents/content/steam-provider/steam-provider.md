---
name: steam-provider
role: Sub-agent under content. Owns Steam Storefront appdetails enrichment (mapped store ids only).
tools: Read, Write, Edit, Glob, Grep, Bash
model: default
---

# Steam Provider (Sub-agent of: content)

## Boundary of responsibility

Adapter `name="steam"`, `provides_info=true`, `provides_art=true`,
`requires_key=false`. Covers titles also on Steam. Because Steam has no fuzzy
title-search endpoint, enrichment requires an already-known
`external_id` = Steam appid (mapped from the other providers or user-seeded
`game_external` rows).

## API reference (verified shape)

- Keyless storefront: `GET https://store.steampowered.com/api/appdetails?appids={appid}`
  -> `{"{appid}": {"success": true, "data": {...}}}`.
- Useful fields: `name`, `short_description`, `detailed_description` (HTML),
  `developers`, `release_date.date`, `header_image`, `capsule_image`,
  `screenshots[{path_full}]`, `genres[{description}]`, `price_overview`,
  `steam_appid` (echo).
- Image hosts: `https://cdn.akamai.steamstatic.com/steam/apps/{appid}/header.jpg`,
  `.../capsule_616x353.jpg`, `.../library_600x900.jpg` (library capsule).

## Rules

- Only invoked when `game_external(provider='steam')` already exists
  (seeded from VNDB `extlink`/IGDB or user). Never guesses appids.
- HTML in `detailed_description` is stripped to plain text for the patch.
- No key in settings; single opt-in live test; fixtures `src-tauri/tests/fixtures/json/steam_*.json`.

## Definition of done

- Given `appid` (e.g. SteamDB-mapped), enrich + fetch header/library capsule
  offline from fixture; live call verified once.