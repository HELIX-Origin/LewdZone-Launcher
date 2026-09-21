# Configuration

> Links between wiki pages are relative and omit the `.md` extension.

## Config file

LewdZone-Launcher stores a JSON config file in the per-OS config dir:

| OS | Path |
| --- | --- |
| Windows | `%APPDATA%/lewdzone/config.json` |
| Linux | `$XDG_CONFIG_HOME/lewdzone/config.json` (or `~/.config/lewdzone/config.json`) |
| macOS | `~/Library/Application Support/lewdzone/config.json` |

## Key settings

```jsonc
{
  "download-root": "D:/Games",            // where downloaded games go
  "dm": "fdm",                            // active download manager
  "site": "https://lewdzone.com",         // base URL (do not change)
  "artwork-cache": true,                  // cache provider artwork locally
  "api-throttle-req-per-sec": 1,          // rate limit for site requests
  "content-providers-enabled": "steamgriddb, vndb, itch",  // active content providers
  "content-priority": "steamgriddb, vndb, igdb, itch, steam, indiedb" // dispatch order
}
```

## Content-provider API keys

The content-provider layer (info + art enrichment) uses several external
sources. Keys are set per-provider from the app's **Settings → API keys** or
the CLI, and are never displayed back:

| Key | Provider | Needed? |
| --- | --- | --- |
| `sgdb-api-key` | SteamGridDB | yes, for icons/grids/heroes/logos |
| `igdb-client-id` | IGDB | yes, for IGDB info + covers |
| `igdb-client-secret` | IGDB | yes (Twitch token exchange) |
| VNDB / itch.io / IndieDB / Steam | — | no keys required (scrape or keyless) |

```sh
lewdzone-launcher settings set sgdb-api-key <key>
lewdzone-launcher settings set content-priority "vndb,igdb,steamgriddb"
```

Keys live in the per-OS config dir (never in the repo; redacted from logs —
see [Security](Security)). Providers that are disabled or missing a key simply
don't enrich — downloads are never affected.

## Changing settings via CLI

```sh
lewdzone-launcher settings get <key>
lewdzone-launcher settings set <key> <value>
```

## Download root

The root under which `Games/` is created; downloads fold into:
`<download-root>/Games/<Title>/<Title> - <Version> - <Platform>[- <Variant>].<ext>`

## Active download manager

```sh
lewdzone-launcher dm list          # show installed managers
lewdzone-launcher dm set active <name>
```

`<name>` is one of `fdm`, `idm`, `torrent`. See
[Download Managers](Download-Managers).

## Database

SQLite database lives in the same config dir as the config file:
`<config-dir>/lewdzone.db`. Configured with WAL journaling, foreign keys ON,
busy timeout 5000ms. See [Architecture](Architecture) → Data model.

## Version

The app version is the single source of truth; it is synced across:
- Tauri manifest (`src-tauri/Cargo.toml` / `tauri.conf.json`)
- Python `__version__` in `lewdzone_launcher/__init__.py`
- Package metadata (`pyproject.toml`)

Sidecar version must equal the app version (enforced at startup). See
[Release Process](Release-Process).