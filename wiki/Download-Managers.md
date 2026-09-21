# Download Managers

> Links between wiki pages are relative and omit the `.md` extension.

## Model

LewdZone-Launcher **never downloads files itself**. The resolver converts a
go-link token into a **real URL**, then a per-manager **adapter** hands that
URL to an installed download manager:

- If no manager is installed, the command fails fast (exit code **4**) and
  lists what's available.
- adapters live in `lewdzone_launcher/services/dm/adapters/`.
- A registry (`MANAGERS`) maps names to adapter instances; the active manager
  is a [Configuration](Configuration) setting (`settings set dm` or `lewdzone-launcher dm set active <name>`).

## Adapter contract

Every adapter implements:

| Member | Purpose |
| --- | --- |
| `name` | `fdm` / `idm` / `torrent` |
| `platforms` | `win32`, `linux`, `darwin`, … |
| `handles_kind` | `http` or `torrent` |
| `detect() -> Path | None` | locate the manager executable |
| `launch(url, target_dir, filename)` | spawn detached, silent |
| `confirm_launch() -> bool` | optional post-check |

## Per-manager invocation

| Manager | Platforms | Kind | Invocation shape |
| --- | --- | --- | --- |
| FDM | Windows | http | `fdm.exe -fs <url>` |
| IDM | Windows | http | `IDMan.exe /d <url> /n /p <target_dir> [/f <filename>]` |
| uTorrent / BitTorrent | Windows, Linux, macOS | torrent | `<client> <magnet-or-local-torrent>` |

Torrents (magnet / `.torrent`) are **never** routed to HTTP managers. For a
`.torrent` URL the tool downloads the small torrent file itself into a job temp
dir, then hands the local path to the client.

## Resolution rule

Only a **resolved real URL** may be handed to a manager. The go-link
`#fragment` is never passed on — naive follow-through would download a redirect
page (see [Security](Security) for the allowlist). Resolved URLs may carry a
trailing literal `\r` that is stripped.

## Folder folding

Completed downloads are folded into:
`<DownloadRoot>/Games/<Title>/<Title> - <Version> - <Platform>[- <Variant>].<ext>`

- Never overwrite an existing file → append `(N)`.
- Multipart parts merge into a `_parts/` folder inside the game folder.
- Path sanitization strips `\ : * ? " < > |` (and `/` where needed).

## Full rule

[Rule 07](../.agents/rules/rule-07-download-manager-integration) in the repo
(`.agents/rules/rule-07-download-manager-integration.md`).