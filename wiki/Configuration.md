# ⚙️ Configuration

> Links between wiki pages are relative and omit the `.md` extension.

## 📄 Config file

LewdZone Launcher stores a JSON config file in the per-OS config dir
(`<config_root>/lewdzone/config.json`, mirrored on the launcher folder layout —
see [Architecture](Architecture) → Folder structure):

| OS | Path |
| --- | --- |
| Windows | `%APPDATA%/lewdzone/config.json` |
| Linux | `$XDG_CONFIG_HOME/lewdzone/config.json` (or `~/.config/lewdzone/config.json`) |
| macOS | `~/Library/Application Support/lewdzone/config.json` |

## 🔑 Key settings

```jsonc
{
  "download-root": "D:/Games",          // where raw downloaded files land (default: per-OS downloads folder)
  "download-grace-seconds": 20,         // pause between download starts (cloud free-tier throttle protection)
  "content-priority": "steamgriddb, vndb, igdb, itch, steam, indiedb", // dispatch order
  "capture-aware": true,                // installer capture heuristics
  "source-priority": "mega, google, dropbox, mediafire", // preferred download-source order
  "theme": "Pink Neon"                  // theme skin name (unset = built-in default)
}
```

> ⚖️ **Download scheduler** — downloads are always dispatched **one at a time**;
> the scheduler waits `download-grace-seconds` (default 20) between each start so
> the site's free-tier mirrors (Mega, Google Drive, MediaFire, ...) aren't
> throttled or blocked. Set it lower on fast connections, higher on flaky ones:

## ⭐ Preferred download sources

By default a game download lists every source available on the game page. The
`source-priority` setting (a comma-separated list of source hosts) reorders the
available sources so your favorites come first:

```sh
lewdzone settings set source-priority "mega, google, dropbox, mediafire"
lewdzone settings get source-priority
```

Valid hosts are the [resolver allowlist](Download-Managers) (`fileknot`,
`transfaze`, `mega`, `google`, `uploadhaven`, `workupload`, `mediafire`,
`dropbox`, `pixeldrain`). Hosts you name that aren't on the game page are
simply skipped — the download never fails just because a source is missing.

## 📥 How downloads are dispatched

There is no download manager to configure. Every resolved URL is dispatched
by host class (see [Downloads & In-App Streaming](Download-Managers)):

- **Direct-file hosts** — streamed in-app with byte progress.
- **Everything else** — opened in the OS default handler (the installed cloud
  app for the service, or the browser). Zero configuration.

## 🎨 Theme skins (ADR-0005)

Themes are first-class: a skin package lives in a **per-OS user-accessible**
skins folder (Windows: next to the executable; macOS/Linux: the app data
folder) at `<skins>/<Name>/theme.json` — one subfolder per theme. Embedded
theme resources live inside the theme folder next to `theme.json` (optional
`assets/` subfolder). Each skin overrides the `--lz-*` design tokens via CSS
custom properties. The built-in default theme is always present; a custom skin
loads in its place when applied. Skins may only carry tokens + assets — never
scripts ([Security](Security)). A malformed skin falls back to the built-in
default theme.

**Nord**, **Dracula**, and **Material** ship with every build and are seeded
into the skins folder on first run — they always appear in the theme list and
double as working reference themes for creators.

Full authoring guidance lives in [Theme development](Theme-Development) —
including a step-by-step walkthrough, the complete token table, manifest
schema, validation rules, and per-OS folder locations.

| Token | Default | Meaning |
| --- | --- | --- |
| `--lz-accent` | `#FF4EC8` | neon magenta accent |
| `--lz-primary` | `#FF5FB2` | neon pink |
| `--lz-cyan` | `#22D3EE` | neon cyan |
| `--lz-bg` | `#0A1118` | deep dark cyan/charcoal |
| `--lz-surface` / `--lz-surface-2` | `#0E1B26` / `#122A3A` | panel surfaces |
| `--lz-text` / `--lz-text-dim` | `#E8F1F8` / `#9AAEC0` | text colors |
| `--lz-danger` | `#FF3B6B` | errors/destructive |
| `--lz-ok` | `#3DFFA2` | success |
| `--lz-radius` | `4px` | corner rounding |
| `--lz-gap` | `12px` | layout spacing |
| `--lz-gradient` | `linear-gradient(160deg, #0A1118 0%, #0E1B26 100%)` | page backdrop |
| `--lz-glow` | `0 0 14px rgba(34, 211, 238, 0.35)` | neon glow |
| `--lz-glass` | `rgba(14, 27, 38, 0.55)` | glassmorphism fill |

```sh
lewdzone settings set theme "Pink Neon"
lewdzone settings get theme
```

Switch skins in **Settings → Appearance** without restarting. The Settings
page lists every skin in the user skins folder (including the seeded Nord/
Dracula/Material defaults); the default theme is always available as
`(default)` regardless of what is installed.

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
lewdzone settings set sgdb-api-key <key> --secret
lewdzone settings set igdb-client-id <id> --secret
lewdzone settings set igdb-client-secret <secret> --secret
lewdzone settings set content-priority "vndb,igdb,steamgriddb"
```

Keys live in the SQLite database's `secret` table (never in `config.json`,
never in the repo; redacted from logs — see [Security](Security)).
`settings get <key>` prints only `(set)` or `(not set)` — never the value.
Providers that are disabled or missing a key simply don't enrich — downloads
are never affected.

## 🛠️ Changing settings via CLI

```sh
lewdzone settings get <key>
lewdzone settings set <key> <value>
```

## 🗂️ Library root

The `library-root` setting relocates both the downloads folder and the installed
apps folder. When set, the launcher uses:

- `<library-root>/downloads/` — raw downloaded files
- `<library-root>/lzapps/` — extracted/installed games

When unset, defaults are per-OS and user-accessible:

| OS | Default downloads | Default lzapps |
| --- | --- | --- |
| Windows | `<install dir>/downloads` | `<install dir>/lzapps` |
| Linux | `<data_root>/downloads` | `<data_root>/lzapps` |
| macOS | `~/Library/Application Support/lewdzone/downloads` | `~/Library/Application Support/lewdzone/lzapps` |

This mirrors the skin-folder pattern: Windows keeps folders next to the
executable; macOS/Linux keep them in the app data directory.

ADR-0005's Steam-style `library/` folder still holds per-game manifests
(`appmanifest_<post_id>.json`), common assets, and artwork; see
[Architecture](Architecture) → Folder structure.

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