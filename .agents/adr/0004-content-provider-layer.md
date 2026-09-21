# ADR-0004: Content-provider enrichment layer

- **Status:** accepted
- **Date:** 2026-09-21
- **Owner:** content family
- **Applies to:** Rule 03, Rule 05, Rule 10

## Context

Many LewdZone pages carry thin metadata (no description, screenshots, release
info, or art) — games by ordinary/indie and amateur developers. The scrape is
the source of truth for *downloads* (versions, platforms, hosts) but often
lacks presentation data. External catalogs (VNDB, IGDB, Steam, itch.io,
IndieDB) can enrich it, but they differ wildly in auth, API shape, and content
policy, and several have no public API at all.

## Decision

- A pluggable **ContentProvider adapter layer** under
  `src/lewdzone_launcher/services/content/providers/` (ADR-0001 rules on
  layers apply):

  ```
  name: str
  provides_info: bool
  provides_art: bool
  requires_key: bool
  search(title) -> list[ProviderCandidate]
  fetch_info(candidate) -> GameInfoPatch | None
  fetch_asset(candidate, kind: AssetKind) -> Path | None
  ```

  `AssetKind = icon | grid | hero | logo | cover | screenshot`.
- **Registry + priority dispatch** (`PROVIDERS`), ordered by settings
  `content-priority` (default `steamgriddb, vndb, igdb, itch, steam, indiedb`);
  stops early once all wanted kinds + info gaps are filled; unconfigured
  key-required providers are skipped silently.
- **Merge rule:** LewdZone scrape is authoritative for downloads; providers
  only fill *missing* info fields (description, developer, release date,
  screenshots, rating, tags, store URL) and supply art — never overwrite a
  scraped download field.
- **Persistence:** `game_external` table maps `post_id → provider →
  external_id` (upsert on `(post_id, provider)`); `artwork_cache` gains
  `provider` + `kind` columns so a game can hold a SteamGridDB icon *and* a
  VNDB cover simultaneously (ADR-0003).
- **Graceful degradation:** provider outage = "no enrichment", never a hard
  failure of listings or downloads.

## Consequences

- **Benefits:** one contract for six v1 providers (+ future); enrichment is
  idempotent, offline-fast after first fetch; settings keys reconciled in the
  app Settings page.
- **Costs/risks:** three providers have no keyless API (IndieDB/itch HTML
  scrape; Steam needs a pre-mapped appid); per-provider rate limits must be
  honored (Rule 05, 1 req/s).
- **Migration:** additive — new providers are just new adapter files +
  registry/settings rows + fakes in `tests/support/`.

## Alternatives considered

1. Only SteamGridDB for art + no info enrichment — rejected: leaves thin
   descriptions/screenshots unresolved; single point of art failure.
2. Merge provider data over scraped fields — rejected: risks corrupting
   download-true data and host/version integrity.

## Verification

- [x] Contract + registry documented in `content.md`, `provider-registry.md`
- [x] Six v1 provider cards authored
- [x] Wiki `Content-Providers` + `Configuration` keys
- [ ] Unit tests: dispatch order, stop-early, zero-gain, missing-key skip (fakes in `tests/support/`)
- [ ] Enrichment e2e for a thin title (VNDB description + SteamGridDB icon + cover, cached + offline replay)