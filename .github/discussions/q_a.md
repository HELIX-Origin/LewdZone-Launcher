# ❓ Frequently Asked Questions (FAQ) & Getting Help

Welcome to the Q&A category! If you have questions about setting up, configuring, or troubleshooting LewdZone Launcher, check the common questions below or start a new thread.

---

### 1. Where do I get the 7-Zip CLI tool for archive extraction?
LewdZone Launcher uses standalone 7-Zip console binaries for maximum extraction speed:
- **Windows**: Download `7z2603-extra.7z` from the [7-Zip extra releases](https://github.com/ip7z/7zip/releases) and point Settings to `7za.exe` (e.g. `C:\Utilities\7z\7za.exe`).
- **Linux**: Download `7z2603-linux-x64.tar.xz` and point Settings to `7zz` (e.g. `/usr/local/bin/7zz`).
- **macOS**: Download `7z2603-mac.tar.xz` and point Settings to `7zz` (e.g. `/usr/local/bin/7zz`).

For step-by-step instructions, see the [Archive Extraction Guide](https://github.com/HELIX-Origin/LewdZone-Launcher/wiki/Archive-Extraction).

### 2. Why do cloud downloads (MEGA, Google Drive, Mediafire) open my browser or cloud app?
Direct file hosts like Fileknot stream in-app with byte-level progress. Cloud file lockers enforce custom web portals, token gates, or client apps, so LewdZone Launcher dispatches them directly to your OS default handler (browser or installed desktop client) without requiring third-party download managers.

### 3. How do I enable SteamGridDB or VNDB artwork enrichment?
Go to **Settings** in the launcher, enter your API key (if required by the provider, such as SteamGridDB), and click **Save**. You can then enrich any game from its library details view or via CLI:
```bash
lewdzone enrich <slug> --provider steamgriddb --json
```

### 4. How do I launch games using the native CLI?
```bash
# List catalog games
lewdzone catalog --page 1 --json

# Resolve a go-link or token
lewdzone resolve <token-or-url> --json

# Scan local library games
lewdzone library --json

# Launch an installed game
lewdzone launch <slug> --json
```

---

Have a question not covered here? Click **New discussion** in Q&A to ask!
