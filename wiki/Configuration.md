# ⚙️ Configuration Reference

> Links between wiki pages are relative and omit the `.md` extension.

LewdZone Launcher manages its state and user preferences through a unified configuration system shared between the Tauri GUI and the native CLI (Rule 03, Rule 13).

---

## 📄 Configuration Files & Storage

Configuration is stored in two locations per operating system:

| OS | Configuration File (`config.json`) | SQLite Database (`lewdzone.db`) |
| --- | --- | --- |
| **Windows** | `%APPDATA%\lewdzone\config.json` | `%APPDATA%\lewdzone\lewdzone.db` |
| **Linux** | `$XDG_CONFIG_HOME/lewdzone/config.json` (`~/.config/lewdzone/config.json`) | `$XDG_DATA_HOME/lewdzone/lewdzone.db` (`~/.local/share/...`) |
| **macOS** | `~/Library/Application Support/lewdzone/config.json` | `~/Library/Application Support/lewdzone/lewdzone.db` |

- **Non-secret settings** are saved as readable JSON in `config.json`.
- **Sensitive credentials** (such as SteamGridDB API keys and IGDB client secrets) are stored encrypted in the SQLite `secret` table and are never written to `config.json` ([Security](Security)).

---

## 🔑 Key Settings Reference

| Key | Type | Default | Description |
| --- | --- | --- | --- |
| `7z-path` | string | `""` | Path to the 7-Zip console executable (`7za.exe`, `7z.exe`, `7zz`, or directory containing them). Alias: `seven-zip-path`. See [Archive Extraction & 7-Zip](Archive-Extraction). |
| `library-root` | string | `""` | Base directory containing `installed/` game directories and optional `games/` custom scan folder. |
| `games-dir` | string | `""` | Folder where you personally extract games. Scanned by the Library view to discover and register games. |
| `download-dir` | string | `""` | Override path for downloaded archives (defaults to the OS downloads folder). |
| `download-grace-seconds`| integer | `20` | Pacing delay (in seconds) between sequential download dispatches to protect against host throttling. |
| `source-priority` | string | `"mega, google, dropbox, mediafire"` | Comma-separated list of preferred download hosts; used to reorder host options in the game detail view. |
| `home-page` | string | `"store"` | Initial view loaded when the application launches (`store`, `library`, `favorites`, `downloads`, `settings`). |
| `theme` | string | `""` | Active skin name (empty string selects the built-in default theme). See [Theme Development](Theme-Development). |
| `capture-aware` | boolean | `true` | When enabled, pauses background network syncing while window capture or streaming is detected. |
| `content-priority` | string | `"steamgriddb, vndb, igdb, itch, steam, indiedb"` | Priority order for querying external metadata providers. |

---

## 🛠️ Configuring via Command Line

Use `lewdzone settings` to inspect and modify any configuration key:

```bash
# View all configuration settings
lewdzone settings get

# Inspect a specific key
lewdzone settings get 7z-path
lewdzone settings get library-root

# Set configuration values
lewdzone settings set 7z-path "C:\Utilities\7z\7za.exe"
lewdzone settings set library-root "G:\LewdZone"
lewdzone settings set games-dir "D:\Games\LewdZone"
lewdzone settings set download-grace-seconds 10
lewdzone settings set home-page "library"
lewdzone settings set theme "Nord"

# Store sensitive API keys securely in SQLite (never written to config.json)
lewdzone settings set sgdb-api-key "your-steamgriddb-key" --secret
lewdzone settings set igdb-client-id "your-twitch-client-id" --secret
lewdzone settings set igdb-client-secret "your-twitch-client-secret" --secret
```

> **GUI Tip:** Sensitive keys like `sgdb-api-key` can also be configured directly in the application under **Settings → Content Providers**. The key is masked, stored in the SQLite `secret` table, and never written to plain-text configuration files.


---

## 🗂️ Library Directory Layout

Setting `library-root` establishes a flat, clean structure on disk:

```text
<library-root>/
├── installed/
│   ├── game-slug/
│   │   ├── app.json
│   │   ├── Game.exe
│   │   └── game_data/
│   └── another-slug/
│       ├── app.json
│       └── Game.exe
└── games/                  # optional custom scan folder
    └── ...
<download-dir>/
├── Game Title [Ongoing] - Version 0.19.1.zip
└── Another Game - Version 1.0.rar
```

- **Downloads:** Raw download archives land in the configured `download-dir`
  (defaults to OS downloads folder). Configure a custom path in Settings.
- **Flat Installs:** Each extracted game resides directly inside `<library-root>/installed/<slug>/`.
- **Legacy Compatibility:** The launcher continues to detect and run existing games located in legacy `<library-root>/lzapps/<slug>/` folders.

---

## 🎨 Theme Skins

Theme packages live in the user-accessible skins directory:
- **Windows:** `<install dir>\skins\<ThemeName>\theme.json`
- **macOS / Linux:** `<data_root>/skins/<ThemeName>/theme.json`

Three reference themes ship with the launcher: **Nord**, **Dracula**, and **Material**. Any custom theme directory containing a valid `theme.json` will automatically appear in the Settings dropdown and can be applied dynamically without restarting the application. Detailed documentation is available in [Theme Development](Theme-Development).

---

## 🔗 Related Pages

- [Archive Extraction & 7-Zip Setup](Archive-Extraction)
- [Downloads & In-App Streaming](Download-Managers)
- [CLI Reference](CLI-Reference)
- [Theme Development](Theme-Development)
- [Security](Security)