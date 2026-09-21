# Getting Started

> Links between wiki pages are relative and omit the `.md` extension.

## Requirements

- **Windows**, **Linux**, or **macOS**
- **Python 3.x** for the command-line engine
- A **download manager** for actual transfers:
  - Windows: Free Download Manager (FDM) or Internet Download Manager (IDM)
  - Any platform: uTorrent / BitTorrent for torrent links
  - If none is installed, download commands fail fast (exit code 4) and list
    what's available.

## Quick start (CLI)

```sh
# install the Python package (from a source checkout)
pip install -e .

# point the tool at your download root
lewdzone-launcher settings set download-root "D:/Games"

# refresh the catalog from LewdZone
lewdzone-launcher sync --json

# find a game
lewdzone-launcher search --query "treasure of nadia" --json

# show a game's versions and download entries
lewdzone-launcher info --game treasure-of-nadia --json

# enqueue a download to your manager of choice
lewdzone-launcher download --game treasure-of-nadia --version latest \
  --platform windows --tab official --manager fdm --json

# list installed games
lewdzone-launcher list --status installed --json
```

## Quick start (desktop app)

Build steps are in [Installing & Building](Installing-and-Building). On first
launch:

1. **Store** page — browse or search; pick a game; choose version/platform;
   enqueue.
2. **Downloads** page — watch live progress; cancel if needed.
3. **Library** — launched/installed games with artwork; right-click for
   Launch, Rebuild shortcuts, Uninstall.
4. **Settings** — download root, active download manager, artwork cache.

## Where things live

- Catalog + download jobs: SQLite database (config dir per OS, see
  [Configuration](Configuration)).
- Download files: `<DownloadRoot>/Games/<Title>/`.
- Artwork cache: `artwork_cache/` next to the database.
- Start-menu / desktop shortcuts: `lewdzone` group (per OS).

## First sync

`sync` fetches catalog data by paging the site's archive. It's rate-limited to
be polite (see [Design Conventions](Design-Conventions) and
[Security](Security)). A full sync can take a while; it resumes incrementally
on later runs.