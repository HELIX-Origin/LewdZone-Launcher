# 🚀 Getting Started

> Links between wiki pages are relative and omit the `.md` extension.

LewdZone Launcher is an all-in-one desktop game launcher and native CLI engine. This guide walks you through requirements, initial configuration, and your first download.

---

## ✅ Requirements

1. **Operating System:** Windows 10/11 (x64/arm64), Linux (x86_64/arm64), or macOS (Intel/Apple Silicon).
2. **7-Zip Console Executable (Recommended):**
   - For fast, multi-threaded extraction of downloaded `.zip`, `.7z`, `.rar`, and `.exe` archives, download the standalone console tool from the **[Archive Extraction & 7-Zip Setup Guide](Archive-Extraction)** (e.g. `7za.exe` for Windows, `7zz` for Linux/macOS).
   - If not installed, basic zip extraction falls back to the native Rust extractor or system `tar`.
3. **Build Dependencies (if building from source):**
   - Rust (stable toolchain)
   - Node.js (v18+) & npm (or pnpm)
   - OS-specific WebView2 / WebKitGTK prerequisites (see [Installing & Building](Installing-and-Building)).

---

## ⚡ Quick Start (CLI)

Build or run the CLI directly from `src-tauri/` or use the installed binary:

```bash
# 1. Check version and help
lewdzone --version
lewdzone --help

# 2. Point launcher to your library root
lewdzone settings set library-root "G:\LewdZone"

# 3. Configure 7-Zip console executable path
lewdzone settings set 7z-path "C:\Utilities\7z\7za.exe"

# 4. Synchronize catalog from LewdZone (incremental)
lewdzone sync --json

# 5. Search or list catalog titles
lewdzone list --limit 10

# 6. View game details, versions, and platforms
lewdzone info treasure-of-nadia

# 7. Download and automatically extract a game
lewdzone download --game treasure-of-nadia --version latest --platform PC --tab official

# 8. List installed games and launch (session playtime automatically tracked)
lewdzone list --library
lewdzone launch treasure-of-nadia

# 9. Create native desktop & Start menu shortcuts
lewdzone shortcuts treasure-of-nadia
```

---

## 🖥️ Quick Start (Desktop App)

1. **Launch the App:** Run `npm run tauri dev` or open the installed application.
2. **Configure Settings:**
   - Go to **Settings** (`/settings`).
   - Set your **Library root** (e.g., `G:\LewdZone`).
   - Set your **7-Zip console executable path** (e.g., `C:\Utilities\7z` or `C:\Utilities\7z\7za.exe`).
   - Choose your favorite **Theme** (Default, Nord, Dracula, or Material).
3. **Browse & Download:**
   - Open **Store** to browse trending games, filter by engine/genre/platform, or search by keyword.
   - Click a game card to view descriptions, screenshots, version dropdowns, and download host sources.
   - Choose your preferred host and click **Download**.
4. **Monitor Downloads:**
   - Head to the **Downloads** view to monitor real-time streaming and extraction progress.
   - You can cancel or delete active and queued items at any time.
5. **Play in Library:**
   - Once extracted, the game appears in the **Library** with its cover art, playtime badges, and last-played info.
   - Click **Launch** to launch the game executable directly from its `app.json` manifest. Play duration is automatically tracked in SQLite.
   - Click the **Shortcut** button next to Launch to create a native desktop or start menu shortcut with the game's embedded icon.
   - Use the **Sort** dropdown to sort games by A–Z, Recently Played, Most Played, or Recently Installed.
   - Use **Scan Games** to discover any games you previously extracted manually into your configured games directory.
6. **System Tray:**
   - Minimizing or closing the window minimizes to the system tray so downloads continue in the background. Right-click the tray icon to quickly navigate to Store, Library, Downloads, Settings, or Quit.

---

## 📁 Where Files Live

- **SQLite Database:** `<data_root>/lewdzone.db` (stores scraped games, tokens, and download queue).
- **Configuration:** `<config_root>/config.json` (persists settings such as `library-root`, `games-dir`, `7z-path`, and `theme`).
- **Secrets:** Stored in the SQLite `secret` table and never written to `config.json` ([Security](Security)).
- **Downloads Directory:** `<library-root>/downloads/<archive>` (stores downloaded archives flat).
- **Installed Directory:** `<library-root>/installed/<slug>/` (contains extracted game files and `app.json`).

---

## 🔗 Related Pages

- [Archive Extraction & 7-Zip Guide](Archive-Extraction)
- [Downloads & In-App Streaming](Download-Managers)
- [Configuration Reference](Configuration)
- [CLI Reference](CLI-Reference)
- [Troubleshooting](Troubleshooting)