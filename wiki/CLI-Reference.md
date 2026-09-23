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
| `list` | List the catalog, the installed library, or the job queue |
| `settings` | Read/write config (`get` / `set`) |
| `shortcuts` | Build/rebuild native shortcuts for installed games |
| `launch` | Launch an installed game |

Note: the previous `download` and `dm` subcommands are merged. `download`
persists; `dm` was removed with the download-manager layer (see
[Download Managers](Download-Managers)).

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

The game is given positionally or via `--game` (flag form). The handler is
chosen automatically by host class — no manager to select:

```
lewdzone download treasure-of-nadia
lewdzone download --game treasure-of-nadia \
  --version latest --platform PC --tab official --json
lewdzone download --game treasure-of-nadia --source mega   # one specific source host
lewdzone download treasure-of-nadia --resume   # resume an existing job
lewdzone download treasure-of-nadia --queue    # enqueue without starting
```

`--source <host>` restricts the download to a single source from that game's
page (e.g. `mega`, `fileknot`, `dropbox`). Without it, every available source
is dispatched in the configured order (see `source-priority` below).

Direct-file hosts (`fileknot`) stream in-app and print `[download] N%`
progress to stderr. Other hosts open in the OS default handler (installed
cloud app or browser). See [Download Managers](Download-Managers).

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
lewdzone settings set download-grace-seconds 20   # pause between download starts
lewdzone settings set source-priority "mega, google, dropbox"  # preferred source order
```

### `dm`

> Removed. Download dispatch no longer uses an active manager — direct-file
> hosts stream in-app, everything else opens in the OS default handler.

## 🧾 Exit codes

| Code | Meaning |
| --- | --- |
| 0 | success |
| 1 | runtime error |
| 2 | usage error |
| 3 | network error |
| 5 | interrupted |

Exit **4** was the download-manager-missing code; it is now unused (gap left
to keep earlier codes stable).

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
lewdzone download --game treasure-of-nadia --version latest \
  --platform PC --tab official --json
lewdzone list --library --json
```

See also [Exit codes & errors](https://github.com/helix-origin/lewdzone-launcher/tree/main/.agents/rules/rule-12-error-handling.md) in the
repo (`.agents/rules/rule-12-error-handling.md`).