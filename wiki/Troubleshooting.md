# 🐛 Troubleshooting

> Links between wiki pages are relative and omit the `.md` extension.

## ⚠️ Exit code 4: download manager missing

`download`/`launch` exits 4 when no supported manager is found.

- Run `lewdzone dm` to see detected managers.
- Install FDM / IDM (Windows) or a torrent client, or set one as your
  [active manager](Download-Managers):
  `lewdzone dm <name>`.
- See [Download Managers](Download-Managers).

## ↩️ Downloads get a redirect page instead of the file

The go-link `#fragment` was passed through instead of being resolved. The tool
must resolve tokens via the site's `start`→`reveal` API and hand only the real
URL to the manager. Re-run with a fresh resolve; report as a bug if it
persists.

## 🐌 Slow or incomplete catalog after `sync`

- Sync is deliberately throttled (~1 req/s). A first full sync is slow by
  design; later runs are incremental.
- Check network (exit 3) and that you're not behind a proxy blocking the site.

## 🧹 `--json` output is polluted

stdout must be pure machine output; diagnostics go to stderr. If you see
warnings/text on stdout, that's a bug — file it.

## 🖼️ Artwork/shortcuts missing

- Ensure the content-provider layer is enabled and keyed:
  `settings get content-priority`, `settings get sgdb-api-key`.
- A SteamGridDB API key (never committed) is needed in
  `~/.config/lewdzone/` or env.
- `.lnk` / `.desktop` / `.app` generation is per-OS; verify the app's shortcut
  group `lewdzone` in your start-menu / desktop / Applications.

## 🔀 App and CLI versions disagree

There is no sidecar: the CLI is the same binary as the app (Rule 13). If
`lewdzone --version` differs from the app version, the binaries were
built from different commits — rebuild both from the same source tree.

## ❓ More help

See [Getting Started](Getting-Started), [Configuration](Configuration), and
open an issue on the
[GitHub repository](https://github.com/helix-origin/lewdzone).