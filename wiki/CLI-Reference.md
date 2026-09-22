# CLI Reference

> Links between wiki pages are relative and omit the `.md` extension. The CLI
> and the desktop app are entry points into the same Rust core: every GUI
> action maps 1:1 to a CLI command.

## Usage

```
lewdzone <command> [options]
```

## Global options

| Option | Meaning |
| --- | --- |
| `--json` | one JSON document per command on stdout |
| `--verbose` | debug logging to stderr |
| `--version` | print version |

## Commands

| Command | Purpose |
| --- | --- |
| `sync` | Refresh catalog from the site (paged, incremental) |
| `search` | Search games (`--query`, `--genre`, `--platform`, …) |
| `info` | Game detail: versions, download entries, metadata |
| `download` | Enqueue a download for the active manager |
| `list` | List catalog / installed games (`--status`) |
| `settings` | Read/write config (`get` / `set`) |
| `shortcuts` | Build/rebuild native shortcuts for installed games |
| `launch` | Launch an installed game |
| `dm` | Manage download-manager adapters (list / set active) |

### `download` flags

```
--game <slug>
--version <version|latest>
--platform <PC|Android|Linux|Mac>
--tab <official|community>
--manager <fdm|idm|torrent>   # default: active manager
--json                        # returns {job_id, status, ...}
```

Torrent links are only accepted by a torrent-capable manager.

## Exit codes

| Code | Meaning |
| --- | --- |
| 0 | success |
| 1 | runtime error |
| 2 | usage error |
| 3 | network error |
| 4 | download manager missing / not found |
| 5 | interrupted |

## Protocol details

- **Query commands:** single JSON doc, e.g.
  `{"games":[...],"count":42}`.
- **Long-running commands:** progress lines go to **stderr**; stdout stays
  machine-clean.
- Every long command with `--json` emits its final result as one JSON doc on
  completion.
- Errors go to **stderr**; stdout stays machine-parseable even on failure.

## Examples

```sh
lewdzone search --query "nad" --json
lewdzone info --game treasure-of-nadia --json
lewdzone download --game treasure-of-nadia \
  --version latest --platform windows --tab official --json
lewdzone list --status installed --json
lewdzone dm list --json
```

See also [Exit codes & errors](https://github.com/helix-origin/lewdzone-launcher/tree/main/.agents/rules/rule-12-error-handling.md) in the
repo (`.agents/rules/rule-12-error-handling.md`).