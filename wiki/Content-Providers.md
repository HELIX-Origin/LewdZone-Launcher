# 🪄 Content Providers & External Enrichment

> Links between wiki pages are relative and omit the `.md` extension.

Not every LewdZone game page carries complete descriptions, high-resolution artwork, or developer information. LewdZone Launcher includes an optional, pluggable **content-provider layer** that enriches listings with external metadata and covers while keeping LewdZone as the authoritative source for downloads and versions.

---

## 🧭 Core Principles

1. **LewdZone is Authoritative for Downloads:** External metadata never overrides or conflicts with official LewdZone version numbers, download tabs, or platform links.
2. **Missing Fields Only:** Providers only supplement fields that are absent or sparse on the LewdZone page (e.g. detailed plot summary, publisher, high-res hero/grid artwork).
3. **Graceful Degradation:** If a provider is unreachable, missing an API key, or encounters an error, the launcher degrades seamlessly without blocking store browsing or downloads.

---

## 🗂️ Supported External Providers

| Provider | ID | Enriched Info | Enriched Artwork | Authentication |
| --- | --- | --- | --- | --- |
| **SteamGridDB** | `steamgriddb` | None | Icons, grids, heroes, logos | Bearer API Key (free on steamgriddb.com) |
| **VNDB (Kana)** | `vndb` | Synopsis, developer, rating, tags, screenshots | Cover, screenshots | Keyless public API |
| **IGDB v4** | `igdb` | Summary, genres, release date, screenshots | High-res covers, backgrounds | Twitch Client-ID & Client Secret |
| **itch.io** | `itch` | Description, author, platform tags | Cover, screenshots | Keyless HTML scrape |
| **Steam Store** | `steam` | Full store description, screenshots | Capsules, header banners | Keyless public storefront API |
| **IndieDB** | `indiedb` | Summary, release status | Promotional artwork | Keyless HTML scrape |

---

## 🔑 Setting Provider Credentials

API keys can be configured directly in **Settings → API keys** or via the CLI:

```bash
# Securely store SteamGridDB key
lewdzone settings set sgdb-api-key "<YOUR_KEY>" --secret

# Securely store IGDB Twitch OAuth credentials
lewdzone settings set igdb-client-id "<YOUR_CLIENT_ID>" --secret
lewdzone settings set igdb-client-secret "<YOUR_CLIENT_SECRET>" --secret

# Set provider query priority order
lewdzone settings set content-priority "steamgriddb, vndb, igdb, itch, steam, indiedb"
```

All credentials are saved in the local SQLite `secret` table and are never printed to console output or logged ([Security](Security)).

---

## 🖼️ Local Artwork Caching

Fetched artwork is stored locally in the library cache:
- Keyed by `(normalized_title, kind)`.
- Reused across launcher restarts and offline sessions.
- Allows lightning-fast library rendering without network latency.

---

## 🔗 Related Pages

- [Architecture](Architecture)
- [Configuration](Configuration)
- [Security](Security)
- [Troubleshooting](Troubleshooting)