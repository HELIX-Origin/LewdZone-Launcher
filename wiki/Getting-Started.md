# 🚀 Getting Started

> Links between wiki pages are relative and omit the `.md` extension.

## ✅ Requirements

- **Windows**, **Linux**, or **macOS**
- **Rust toolchain** (stable) + **Node.js/npm** when building from source
- No download manager needed. Direct-file hosts (`fileknot`) stream in-app;
  cloud-host pages open in the OS default handler — the installed desktop app
  for that service (MEGA, Google Drive, Dropbox, ...) or the browser if none is
  installed.

## ⚡ Quick start (CLI)

```sh
# build the CLI (from a source checkout) — see Installing & Building
cargo build --release          # from src-tauri/
./target/release/lewdzone --help

# point the tool at your download root
lewdzone settings set download-root "D:/Games"

# refresh the catalog from LewdZone
lewdzone sync --json

# browse the archive (or list the synced catalog)
lewdzone search --json
lewdzone list

# show a game's versions and download entries
lewdzone info --game treasure-of-nadia --json

# choose the download source and stream / open the download
lewdzone download --game treasure-of-nadia --version latest \
  --platform PC --tab official --json

# list installed/library games
lewdzone list --library --json
```

## 🖥️ Quick start (desktop app)

Build steps are in [Installing & Building](Installing-and-Building). On first
launch:

1. **Store** page — browse or search; pick a game; choose version/platform;
   enqueue.
2. **Downloads** page — watch live progress; cancel if needed.
3. **Library** — launched/installed games with artwork; right-click for
   Launch, Rebuild shortcuts, Uninstall.
4. **Settings** — download root, preferred sources, artwork cache, and
   content-provider API keys (SteamGridDB / IGDB etc.).

## 📁 Where things live

- Catalog + download jobs: SQLite database in the data dir per OS
  (e.g. `%APPDATA%\lewdzone\lewdzone.db`; see [Configuration](Configuration)).
- Download files: `<DownloadRoot>/Games/<Title>/`.
- Installed library: `<data_root>/library/` with
  `common/<Title>/`, `downloads`, and `artwork/` (see
  [Downloads & Streaming](Download-Managers)).
- Start-menu / desktop shortcuts: `lewdzone` group (per OS).

## 🔄 First sync

`sync` fetches catalog data by paging the site's archive. It's rate-limited to
be polite (see [Design Conventions](Design-Conventions) and
[Security](Security)). A full sync can take a while; it resumes incrementally
on later runs.