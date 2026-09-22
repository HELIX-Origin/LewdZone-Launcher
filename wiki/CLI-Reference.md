# 💻 CLI Reference

> Links between wiki pages are relative and omit the `.md` extension. The CLI
> and the desktop app are entry points into the same Rust core: every GUI
> action maps 1:1 to a CLI command.

## 🖥️ Usage

```
lewdzone <command> [options]
```

## 🎛️ Global options

| Option | Meaning |
| --- | --- |
| `--json` | one JSON document per command on stdout |
| `-v`, `--verbose` | debug logging to stderr |
| `--no-color` | disable colorized output |
| `--db <PATH>` | override the SQLite catalog path |
| `--config <PATH>` | override the config file path |
| `--version` | print the app version |
| `--help` | print command help |

## 📟 Commands

| Command | Purpose |
| --- | --- |
| `sync` | Refresh catalog from the site (paged, incremental; `--full` resyncs) |
| `search` | Browse the archive (Popularity); free-text search not yet wired |
| `info` | Game detail: versions, download entries, metadata |
| `download` | Dispatch a resolved URL to the active download manager |
| `list` | List the catalog, the installed library, or the job queue |
| `settings` | Read/write config (`get` / `set`) |
| `shortcuts` | Build/rebuild native shortcuts for installed games |
| `launch` | Launch an installed game |
| `dm` | List download managers or select the active one |

### `sync`

```
lewdzone sync                # incremental (resumes from last synced page)
lewdzone sync --full         # resync every archive page
lewdzone sync --platform PC  # filter platform (PC | Android | Linux | Mac)
```

### `info`

```
lewdzone info treasure-of-nadia            # summary
lewdzone info --game treasure-of-nadia     # flag form (same thing)
lewdzone info 18212                        # accepts slug, post id, or URL
lewdzone info treasure-of-nadia --versions # full version + entry JSON
```

### `download`

The game is given positionally or via `--game` (flag form). The manager is
**not** chosen here — the **active manager** (set with `dm <name>`) receives
the download:

```
lewdzone download treasure-of-nadia
lewdzone download --game treasure-of-nadia \
  --version latest --platform PC --tab official --json
lewdzone download treasure-of-nadia --resume   # resume an existing job
lewdzone download treasure-of-nadia --queue    # enqueue without starting
```

Torrent links are only accepted by a torrent-capable manager (exit 4
otherwise).

### `list`

```
lewdzone list            # catalog from the SQLite DB
lewdzone list --library  # installed/library games
lewdzone list --jobs     # job queue
```

### `settings`

```
lewdzone settings get              # all settings
lewdzone settings get download-root
lewdzone settings set download-root "D:/Games"
```

### `dm`

```
lewdzone dm            # list detected managers + the active one
lewdzone dm fdm        # select FDM as the active manager
```

## 🧾 Exit codes

| Code | Meaning |
| --- | --- |
| 0 | success |
| 1 | runtime error |
| 2 | usage error |
| 3 | network error |
| 4 | download manager missing / not found |
| 5 | interrupted |

## 🛠️ Protocol details

- **Query commands:** single JSON doc, e.g.
  `{"games":[...],"count":42}`.
- **Long-running commands:** progress lines go to **stderr**; stdout stays
  machine-clean.
- Every long command with `--json` emits its final result as one JSON doc on
  completion.
- Errors go to **stderr**; stdout stays machine-parseable even on failure.

## ✨ Examples

```sh
lewdzone sync --json
lewdzone info --game treasure-of-nadia --json
lewdzone dm
lewdzone dm fdm
lewdzone download --game treasure-of-nadia --version latest \
  --platform PC --tab official --json
lewdzone list --library --json
```

See also [Exit codes & errors](https://github.com/helix-origin/lewdzone-launcher/tree/main/.agents/rules/rule-12-error-handling.md) in the
repo (`.agents/rules/rule-12-error-handling.md`).