---
name: enrich-game-and-art
description: Enrich a game with external metadata and cached artwork
---

# Skill: Enrich Game and Art

## When to use

A game card or detail page needs external metadata (description, developer,
rating, genres, screenshots) or cached artwork (icon / cover / background).

## Owner agent families

- [content](../agents/content/content.md)
- [gui](../agents/gui/gui.md)

## Inputs

- `GameCard` (slug, title, post_id, thumb_url, genres, etc.)
- Optional API keys in SQLite `secret` table (`sgdb-api-key`, `igdb-client-id`,
  `igdb-client-secret`).

## Outputs

- `Enrichment` JSON for metadata.
- Cached artwork file path converted to a `file://` URL.

## Rules

1. **LewdZone is the default source.** Only use external provider data when the
   LewdZone scraped field is missing or explicitly inferior.
2. **Providers are best-effort.** A 404 or missing key must not fail the whole
   pipeline; log and continue.
3. **Secrets never echo.** Read API keys from the SQLite `secret` table; surface
   only presence markers in the UI.
4. **Artwork cache keyed by title.** Use `content::cache_key(title)` and store
   files under the artwork cache dir; record hits in `artwork_cache`.

## Steps

1. Build or receive a `GameCard` from the scraped catalog / detail page.
2. Call `core::content::enrich(ctx, &card)` to obtain merged metadata.
3. Merge results into the UI model, preserving LewdZone values where present.
4. For artwork, call `core::content::artwork(ctx, &card, kind)` (or the
   `artwork_url` Tauri command), which returns a local `file://` URL.
5. If LewdZone has a `thumb_url` and `kind == Cover`, the command returns that
   URL directly; external providers are queried only for missing covers or for
   non-cover kinds.

## Verification checklist

- [ ] `content_enrich` returns baseline fields when no providers are enabled.
- [ ] `artwork_url` returns `card.thumb_url` for cover when present.
- [ ] Provider failures are logged but do not surface as UI errors.
- [ ] Cached artwork file exists after first fetch.
- [ ] Gates: `cargo test`, `npm run check`, `npm run test`.

## Related

- Agent: `.agents/agents/content/content.md`
- Rule: `.agents/rules/rule-10-security.md`
- Template: `.agents/templates/content-provider.md`
- Implementation: `src-tauri/src/core/content/mod.rs`
