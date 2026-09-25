# 💻 CLI Reference

> Links between wiki pages are relative and omit the `.md` extension. The CLI
> and the desktop app are two entry points into the same Rust core: every GUI
> action maps 1:1 to a CLI command (Rule 03, Rule 13).

---

## 🖥️ Usage

```bash
lewdzone <command> [options]
```

---

## 🎛️ Global Options

| Option | Description |
| --- | --- |
| `--json` | Output machine-readable JSON on stdout. |
| `-v`, `--verbose` | Enable verbose diagnostic logging to stderr. |
| `--no-color` | Disable ANSI color codes in terminal output. |
| `--db <PATH>` | Override the default SQLite catalog path. |
| `--config <PATH>` | Override the default JSON configuration path. |
| `--version` | Display application and CLI version. |
| `--help` | Print command usage and help. |

---

## 📟 Commands

| Command | Purpose |
| --- | --- |
| `sync` | Fetch and update catalog records from lewdzone.com (incremental by default; `--full` resyncs all). |
| `search` | Search titles or browse archive pages. |
| `info` | Display comprehensive metadata, version list, platforms, and download host links for a game. |
| `download` | Resolve download token, stream direct-file hosts in-app, extract with 7-Zip CLI, or dispatch to OS handler. |
| `list` | Query the catalog, installed game library, or the download queue. |
| `settings` | Read and write persistent configuration settings (`get` / `set`). |
| `shortcuts` | Create native per-OS desktop and start menu shortcuts with embedded game icons. |
| `launch` | Launch an installed game executable, recording play count, last played timestamp, and session playtime. |
| `favorites` | Manage favorite games list (`list`, `add`, `remove`). |

---

### `sync`

Synchronizes catalog data from LewdZone. Respects network etiquette (~1 req/s) and resumes from the last known page.

```bash
lewdzone sync                        # Incremental sync (continues where previous sync left off)
lewdzone sync --full                 # Resyncs all pages from page 1
lewdzone sync --platform PC          # Filter to specific platform (PC, Linux, Mac)
lewdzone sync --platform PC --json   # Emits structured JSON summary upon completion
```

---

### `info`

Retrieves metadata, download tabs, versions, and platforms for a game. Accepts slug, numeric post ID, or full lewdzone URL.

```bash
lewdzone info treasure-of-nadia
lewdzone info --game treasure-of-nadia --versions --json
lewdzone info 18212
```

---

### `download`

Resolves download tokens and processes game archives:

```bash
# Download latest version for PC from the official tab
lewdzone download --game treasure-of-nadia --version latest --platform PC --tab official

# Choose a specific host source
lewdzone download --game treasure-of-nadia --source fileknot

# Enqueue without immediate start
lewdzone download --game treasure-of-nadia --queue

# Resume an existing download job
lewdzone download --game treasure-of-nadia --resume
```

- **Direct hosts (`fileknot`, `pixeldrain`, `mediafire`, `workupload`):** Downloaded directly into the configured `download-dir` with real-time percentage progress printed to stderr, then automatically decompressed into `<library-root>/installed/<slug>/` using the configured 7-Zip CLI ([Archive Extraction & 7-Zip](Archive-Extraction)).
- **Cloud hosts:** Dispatched to the OS default browser or desktop cloud client.

---

### `list`

Lists database contents, installed games, or queued jobs:

```bash
# List catalog entries
lewdzone list --limit 20
lewdzone list --search "hotel"

# List installed games in the library (reads app.json manifests)
lewdzone list --library
lewdzone list --library --json

# List current download and extraction queue
lewdzone list --jobs
```

---

### `settings`

Reads or writes launcher configuration keys:

```bash
# View all settings as JSON
lewdzone settings get

# Inspect specific keys
lewdzone settings get 7z-path
lewdzone settings get library-root

# Update settings
lewdzone settings set 7z-path "C:\Utilities\7z\7za.exe"
lewdzone settings set library-root "G:\LewdZone"
lewdzone settings set games-dir "D:\Games\LewdZone"
lewdzone settings set download-grace-seconds 15

# Securely set API keys (persisted in SQLite secrets table)
lewdzone settings set sgdb-api-key "<YOUR_KEY>" --secret
```

---

### `shortcuts`

Generates working native operating system desktop and start menu shortcuts for an installed game:
- **Windows:** `.lnk` shortcut files generated with Windows Script Host automation, targeted directly to the game binary with the embedded `.exe` icon index.
- **Linux:** FreeDesktop `.desktop` entry files placed in `~/Desktop/` and `~/.local/share/applications/`.
- **macOS:** Native executable command aliases located on `~/Desktop/`.

```bash
# Create desktop and start menu shortcuts
lewdzone shortcuts treasure-of-nadia
lewdzone shortcuts --game harem-hotel
```

---

### `launch`

Launches an installed game by reading `<installed>/<slug>/app.json` (or legacy `<lzapps>/<slug>/app.json`). Spawns the executable candidate safely without shell injection. Also records game launch history, increments `play_count`, updates `last_played_at`, and asynchronously computes session duration upon process exit to accumulate `playtime_seconds`.

```bash
lewdzone launch treasure-of-nadia
lewdzone launch --game harem-hotel
```

---

### `favorites`

Manages games marked as favorites for quick access in both GUI and CLI:

```bash
# List all favorited games
lewdzone favorites list
lewdzone favorites list --json

# Add a game to favorites
lewdzone favorites add wild-life

# Remove a game from favorites
lewdzone favorites remove wild-life
```

---

## 🧾 Exit Codes

| Code | Meaning | Description |
| --- | --- | --- |
| `0` | Success | Command completed successfully. |
| `1` | Runtime Error | File I/O, extraction error, or database failure. |
| `2` | Usage Error | Invalid arguments, unknown command, or missing required parameter. |
| `3` | Network Error | Connection timeout, HTTP failure, or rate limit encounter. |
| `4` | *(unused — reserved)* | Formerly download-manager-missing; kept open for script compatibility. |
| `5` | Interrupted | User cancelled the operation (`SIGINT` / Ctrl+C). |

---

## 🛠️ Machine Protocol (`--json`)

- **Standard Output (`stdout`):** Dedicated exclusively to machine-readable JSON data when `--json` is supplied.
- **Diagnostic Output (`stderr`):** Progress bars, logs, and human-readable diagnostics are routed to `stderr`.
- **Parsing Contract:** Automation scripts and GUI callers parse `stdout` safely without interference from logs.

---

## 🔗 Related Pages

- [Architecture](Architecture)
- [Downloads & In-App Streaming](Download-Managers)
- [Archive Extraction & 7-Zip Guide](Archive-Extraction)
- [Configuration](Configuration)