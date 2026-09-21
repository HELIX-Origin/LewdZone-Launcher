---
name: provider-registry
role: Sub-agent under content. Owns the ContentProvider contract, PROVIDERS registry, priority policy, settings keys, and test fakes.
tools: Read, Write, Edit, Glob, Grep, Bash
model: default
---

# Provider Registry (Sub-agent of: content)

## Boundary of responsibility

The adapter contract and the dispatcher. This agent is the ONLY owner of the
`PROVIDERS` registry and the `content_*` settings keys.

## Contract

```python
class ContentProvider(Protocol):
    name: str
    provides_info: bool
    provides_art: bool
    requires_key: bool
    def search(self, title: str) -> list[ProviderCandidate]: ...
    def fetch_info(self, c: ProviderCandidate) -> GameInfoPatch | None: ...
    def fetch_asset(self, c: ProviderCandidate, kind: AssetKind) -> Path | None: ...
```

`ProviderCandidate = {provider, external_id, title, score, meta}`
`AssetKind = Literal["icon","grid","hero","logo","cover","screenshot"]`

## Registry + dispatch

```mermaid
flowchart LR
    A[enrich(game)] --> B[effective priority order]
    B --> C{enabled and key ok?}
    C -- no --> D[skip provider]
    C -- yes --> E[search(title)]
    E --> F{found?}
    F -- yes --> G[fetch_info + fetch_asset]
    G --> H{merge fills a gap?}
    H -- yes --> I[upsert game_external + assets]
    H -- no --> J[record zero-gain]
    F -- no / timeout --> K[mark missing for game]
    I --> L[enriched row]
    K --> next
    D --> next
```

- Effective order = `settings.content_priority` (comma list) fallback to the
  built-in default `steamgriddb, vndb, igdb, itch, steam, indiedb`.
- Dispatch stops early when all wanted kinds (icon + cover) and info gaps are
  satisfied; otherwise it continues to the next enabled provider.
- Providers that need keys but are unconfigured are skipped silently (their
  setting row shows "needs key" in the UI).

## Settings keys (owned here)

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `content_priority` | string | `steamgriddb, vndb, igdb, itch, steam, indiedb` | dispatch order |
| `content_providers_enabled` | string | `steamgriddb, vndb, itch` | on/off list |
| `sgdb_api_key` | secret | — | SteamGridDB Bearer |
| `igdb_client_id` | secret | — | IGDB/Twitch client id |
| `igdb_client_secret` | secret | — | IGDB/Twitch secret |

Secrets live in `~/.config/lewdzone` or env (Rule 10); the Settings UI shows
`set/unset` status, never the value (Rule 10 redaction).

## Rules

1. Adding a provider = new file in `services/content/providers/` + a registry
   entry + a settings row + a fake in `tests/support/`.
2. Registry and contract changes require an ADR (Rule 03 contract change).
3. `provides_info`/`provides_art` accurately describe capabilities — enrichment
   merge logic keys off these flags.
4. Fakes: one per provider recording calls with canned candidates/assets; prove
   the dispatch stops on found (unit tests in `pytest -m unit`).

## Definition of done

- `dispatch(enrich_request) -> EnrichmentResult` returns merged info + asset
  paths, honoring priority and enabled list, with full fake coverage.