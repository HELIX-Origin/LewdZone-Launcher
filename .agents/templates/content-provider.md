---
name: content-provider
description: Template for adding a new external content-provider adapter
---

# New Content Provider: {{ ProviderName }}

## Identity

- **Provider slug:** `{{ slug }}` (e.g. `steamgriddb`, `vndb`, `igdb`)
- **Data provided:** {{ description, genres, screenshots, rating, etc. }}
- **Artwork provided:** {{ icon / cover / background / none }}
- **API key required:** {{ yes/no; if yes, secret key name }}

## API / scraping notes

- Base URL: `{{ https://example.com/api }}`
- Auth: {{ Bearer token in `Authorization` header / Twitch client-credentials / none }}
- Rate limits: {{ requests per window }}
- Search endpoint: `{{ ... }}`
- Detail / artwork endpoints: `{{ ... }}`

## Implementation checklist

- [ ] Add `src-tauri/src/core/content/{{ slug }}.rs`.
- [ ] Implement `Provider` trait:
  - `name()` returns `"{{ slug }}"`.
  - `enabled()` returns true when required secrets are present.
  - `enrich()` returns `Result<Option<Enrichment>, Error>`.
  - `artwork()` returns `Result<Option<Vec<u8>>, Error>`.
- [ ] Map provider fields to `Enrichment`:
  - `description` from {{ field }}
  - `developer` from {{ field }}
  - `rating` from {{ field }}
  - `tags` from {{ tags/genres }}
  - `genres` from {{ actual genres, if distinct }}
  - `screenshots` from {{ field }}
- [ ] Register provider in `content/mod.rs` `providers()` list.
- [ ] Add secret key to `SECRET_KEYS` in `content/mod.rs` if required.
- [ ] Add unit tests with offline fixtures / stubs.
- [ ] Update `wiki/Content-Providers.md` with the new row.

## Verification

- [ ] `cargo test` passes.
- [ ] Provider is skipped when its secret is missing.
- [ ] Provider 404s/empty results do not break enrichment for other providers.
