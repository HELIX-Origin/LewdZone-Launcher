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
  "download-root": "D:/Games",        // where downloaded games go
  "dm": "fdm",                        // active download manager
  "site": "https://lewdzone.com",     // base URL (do not change)
  "artwork-cache": true,              // cache SteamGridDB artwork
  "api-throttle-req-per_sec": 1       // rate limit for site requests
}
```

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