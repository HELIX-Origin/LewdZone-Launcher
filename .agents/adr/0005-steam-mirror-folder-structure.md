# ADR-0005: Steam-mirror app folder structure

- **Status:** accepted
- **Date:** 2026-09-21
- **Owner:** gui, cli, cli/command-designer
- **Applies to:** Rule 03, Rule 13

## Context

The product is a **desktop game launcher**. The user's standing directive: the
app's internal on-disk structure must be **more or less identical to the Steam
client** — "the only difference being the color scheme and the fact that it is
being built for the LewdZone.com website instead of for steam." Reference
installation inspected at `D:\Games\Steam`.

Steam's data model (observed):

| Steam concept | Steam on-disk layout |
|---|---|
| Steam root | `steam/` (binary, `steamapps/`, `config/`, `userdata/`, `logs/`, `appcache/`) |
| Per-game manifest | `steamapps/appmanifest_<appid>.acf` — AppState{appid, name, installdir, StateFlags, LastUpdated, LastPlayed, SizeOnDisk, buildid, DownloadType, BytesToDownload/BytesDownloaded, ...} |
| Installed games | `steamapps/common/<Name>/` — one folder per product, folder name = installdir |
| Active downloads | `steamapps/downloading/<AppId>/` |
| Shader cache | `steamapps/shadercache/<appid>/` |
| Library roots registry | `steamapps/libraryfolders.vdf` — ordered roots, each with `path`, `label`, `apps{<appid>: size}` |
| Per-user data | `userdata/<steamid3>/` — `config/`, `gamerecordings/`, `ugc/` |
| Grid artwork cache | `userdata/<steamid3>/config/grid/<appid>.jpg`, `<appid>p.jpg`, `<appid>_hero.jpg`, `<appid>_logo.png` (header / portrait / hero / logo) |
| Settings | `config/config.vdf` |
| Machine-local cache | `%LOCALAPPDATA%\Steam\` — `htmlcache/`, `local.vdf` |
| Per-game saves | `<Documents>\My Games\<Game>\` |
| Logs | `logs/<component>.txt` — one file per subsystem |
| Skins / themes | classic Steam `skins/<Name>/` — user-installed custom themes; **removed by Valve in the modern client, retained here per user directive ("keep the theme support that was dropped")** |

## Decision

Our launcher mirrors that shape 1:1, renaming only the product-specific leaf:

```
<DATA_ROOT>/lewdzone/                     # the "steam root" (default per OS config dir)
  lewdzone.db                            # SQLite catalog (tokens, jobs, artwork index)
  config.json                            # settings (mirror of config.vdf)
  appcache/                              # http + provider cache
  logs/                                  # one <component>.log per subsystem
  library/                              # the "steamapps/" analog
    libraryfolders.json                 # ordered library roots { id: { path, label, games {post_id: size} } }
    appmanifest_<post_id>.json          # per-game manifest (AppState analog: name, installdir,
                                        #   state_flags, last_updated, last_played, size_on_disk,
                                        #   version_label, hosts, bytes_to_download, bytes_downloaded)
    common/<Game Title>/                # installed games — folder name = game title (installdir analog)
    downloading/<post_id>/              # in-progress downloads
    artwork/<post_id>_hero.png          # grid artwork cache (Steam grid analog; files, not just
            ..._logo.png                   SQLite rows — artwork_cache.path points here)
  userdata/<local_user_id>/             # per-user profile dir (one active profile for now)
    config.json                         # per-profile overrides
    shortcuts/                          # per-OS native shortcut records

<CACHE_ROOT>/lewdzone/                   # machine-local cache (Steam %LOCALAPPDATA%\Steam analog)
  htmlcache/                            # webview/http cache
  local.vdf -> local.json               # machine-local state (window pos, last library index)
<DOCUMENTS>/My Games/<Game Title>/      # per-game saves (Steam Documents\My Games analog)
<SKINS_ROOT>/<Name>/                   # user-installed theme skins (retained Steam feature), one subfolder per theme
  <SKINS_ROOT> per OS                 # Windows: <install dir>/skins; macOS/Linux: <DATA_ROOT>/lewdzone/skins
  theme.json                          # manifest + --lz-* tokens; assets/ beside it (embedded resources)
  Nord/, Dracula/, Material/          # bundled reference themes, seeded on first run
```

- **Download root key** in settings selects the library root(s); default library
  folder lives under `<DATA_ROOT>/lewdzone/library`. A second installed drive =
  an additional `libraryfolders.json` entry, exactly like Steam.
- **The app manifest is the source of truth for "is a game installed".** It is
  written atomically on install/update/verify; the SQLite `download_job` rows
  stay the task history.
- **Artwork files** (covers/heroes/logos/icons) are cached under
  `library/artwork/<post_id>*.png` and indexed in SQLite `artwork_cache`
  (ADR-0004) so a game can hold multiple provider kinds simultaneously.
- Filenames mirror Steam suffixes: `_hero`, `_logo`, `p` (portrait/cover),
  bare name (header).
- **Cache goes to `%LOCALAPPDATA%`** (Windows), `~/Library/Caches` (macOS),
  `$XDG_CACHE_HOME` (Linux); **per-game saves go to `Documents\My Games`** —
  exactly what Steam does. Cache is expendable; saves are user data.
- **Themes are a first-class citizen.** A per-OS user-accessible skins folder
  (Windows: `<install dir>/skins`; macOS: `~/Library/Application Support/
  lewdzone/skins`; Linux: `$XDG_DATA_HOME/lewdzone/skins`), one subfolder per
  theme, holds user-installed theme packages (a `theme.json` manifest + `--lz-*`
  token overrides; embedded resources sit inside the theme folder next to the
  manifest). The built-in default theme is compiled in and never stored there;
  custom skins load in its place at runtime. The three bundled reference themes
  (Nord, Dracula, Material) are embedded in the binary, seeded into the skins
  folder on first run, and double as live wiki examples. The webview loads
  tokens from the active skin at startup; Settings has a Theme picker. This
  deliberately **retains the custom-skin capability Valve dropped** (user
  directive).

## Consequences

- **Benefits:** the folder layout already proven by Steam maps cleanly to a
  launcher; multi-drive libraries come free via `libraryfolders.json`; app
  manifest enables offline "is installed / needs update" checks without a DB;
  cache separation (`LOCALAPPDATA`) and user-data separation (Documents/My
  Games) match OS expectations; theme skins restore a capability users lost.
- **Costs/risks:** we do not replicate Steam's checksum/depot machinery — the
  manifest is a light metadata record; migrations only add fields; skin
  packages must be sandboxed (CSS tokens only, no arbitrary JS) to keep the
  webview safe (Rule 10).
- **Migration:** forward-only — new fields appended to the manifest schema;
  ADR-0003 migrations unchanged.

## Verification

- [ ] `paths.rs` exposes the mirrored tree (data root, library, common,
      downloading, artwork, userdata, logs, cache, my-games) with per-OS
      defaults
- [ ] libraryfolders get/set in settings; a second library root adds/removes
      an entry without data copy
- [ ] install writes `appmanifest_<id>.json` atomically; sqlite unchanged
- [ ] artwork fetch writes `<root>/library/artwork/<id>_<kind>.png` and indexes
      the path in `artwork_cache`
- [ ] `list --library` reads manifests (offline); UI Library page
      renders grid from manifests + artwork
- [ ] Theme picker applies a user-installed skin from `skins/` and resets to
      default without app restart when tokens change