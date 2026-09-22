# ⚙️ Configuration

> Links between wiki pages are relative and omit the `.md` extension.

## 📄 Config file

LewdZone Launcher stores a JSON config file in the per-OS config dir
(`<config_root>/lewdzone/config.json`, mirrored on the Steam layout —
see [Architecture](Architecture) → Folder structure):

| OS | Path |
| --- | --- |
| Windows | `%APPDATA%/lewdzone/config.json` |
| Linux | `$XDG_CONFIG_HOME/lewdzone/config.json` (or `~/.config/lewdzone/config.json`) |
| macOS | `~/Library/Application Support/lewdzone/config.json` |

## 🔑 Key settings

```jsonc
{
  "download-root": "D:/Games",          // library root (ADR-0005), default <data_root>/library
  "dm": "fdm",                          // active download manager
  "content-priority": "steamgriddb, vndb, igdb, itch, steam, indiedb", // dispatch order
  "capture-aware": true,                // installer capture heuristics
  "theme": "Pink Neon"                  // theme skin name (unset = built-in default)
}
```

## 🎨 Theme skins (ADR-0005)

Themes are first-class: a skin package lives at
`<config_root>/lewdzone/skins/<Name>/theme.json` and overrides the
`--lz-*` design tokens via CSS custom properties (optional `assets/` folder).
Skins may only carry tokens + assets — never scripts ([Security](Security)).
A malformed skin falls back to the built-in default theme.

| Token | Default | Meaning |
| --- | --- | --- |
| `--lz-accent` | `#CB3D80` | site magenta accent |
| `--lz-primary` | `#BC2A5E` | hot pink (icon two-tone) |
| `--lz-cyan` | `#32B6CD` | cyan (icon two-tone) |
| `--lz-bg` | `#14121A` | near-black purple tint |
| `--lz-surface` / `--lz-surface-2` | `#1F1B28` / `#2A2434` | panel surfaces |
| `--lz-text` / `--lz-text-dim` | `#F4F1F6` / `#BDB3C6` | text colors |
| `--lz-danger` | `#E5484D` | errors/destructive |
| `--lz-ok` | `#46D88B` | success |

```sh
lewdzone settings set theme "Pink Neon"
lewdzone settings get theme
```

Switch skins in **Settings → Appearance** without restarting.

## 🗝️ Content-provider API keys

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
lewdzone settings set sgdb-api-key <key>
lewdzone settings set content-priority "vndb,igdb,steamgriddb"
```

Keys live in the per-OS config dir (never in the repo; redacted from logs —
see [Security](Security)). Providers that are disabled or missing a key simply
don't enrich — downloads are never affected.

## 🛠️ Changing settings via CLI

```sh
lewdzone settings get <key>
lewdzone settings set <key> <value>
```

## 🗂️ Library root

ADR-0005 mirrors Steam's multi-root library: the active library root is chosen
by the `library-root` setting (default `<data_root>/library`, i.e.
`%APPDATA%\lewdzone\library` on Windows).
Per-game manifests (`appmanifest_<post_id>.json`) are the source of truth for
installed games; see [Architecture](Architecture) → Folder structure.

## 🚚 Active download manager

```sh
lewdzone dm          # list detected managers + the active one
lewdzone dm <name>   # select the active manager
```

`<name>` is one of `fdm`, `idm`, `torrent`. See
[Download Managers](Download-Managers).

## 🗄️ Database

SQLite database lives at `<data_root>/lewdzone.db` (e.g.
`%APPDATA%\lewdzone\lewdzone.db` on Windows).
Configured with WAL journaling, foreign keys ON, busy timeout 5000ms. See
[Architecture](Architecture) → Data model.

## 🔖 Version

The app version is the single source of truth; it is synced across:
- `src-tauri/Cargo.toml` (crate version + `tauri.conf.json`)
- `package.json`

The CLI reports the same version as the app (`lewdzone --version`).
See [Release Process](Release-Process).