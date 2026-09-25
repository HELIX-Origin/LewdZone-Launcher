# 📥 Downloads, Streaming & Extraction

> Links between wiki pages are relative and omit the `.md` extension.

LewdZone Launcher provides an integrated download pipeline that resolves download tokens, streams direct files in-app, dispatches cloud hosts, and automatically extracts multi-format game archives using the **7-Zip console executable**.

---

## 🧠 Download Architecture & Host Classes

When a user initiates a download, the launcher takes the go-link token (`#t=v1...`), queries the LewdZone `start` → `reveal` API, and acts based on the resolved URL's host class:

| Host Class | Examples | Handling |
| --- | --- | --- |
| **Direct-file hosts** | `fileknot`, `pixeldrain`, `mediafire`, `workupload` | **Streamed in-app** with real-time byte progress, downloaded directly to `<downloads>/<archive>`, and automatically unpacked into `<installed>/<slug>/`. |
| **Cloud storage & file hosts** | `mega`, `google`, `dropbox`, `uploadhaven` | **Dispatched to OS default handler** (installed native desktop client or default web browser). |
| **Verification-required links** | Go-links requiring countdown or captcha | Opened in the launcher's **in-app sandboxed webview resolver**, completely isolating third-party trackers, popups, and malicious scripts. |

> 🚫 **Platform notice:** Android downloads are not supported on the desktop launcher. The store UI filters them out, and CLI attempts will return an explicit error.

---

## 🚦 Dispatch & Security Rules

1. **Tokens stored, never URLs:** Database records store ephemeral go-link tokens rather than real URLs ([Security](Security)).
2. **Blocked hosts blacklist:** Rather than restricting downloads to a narrow allowlist, the launcher maintains a blacklist of blocked hosts (`gofile`, `gofiles`, `zippyshare`, `cdnclick`, `anonfile`, `anonfiles`, `anonzip`, `uptobox`, `yourfilestore`, `qiwi`, `transfersh`). All other functional mirrors are permitted.
3. **Same-host redirect constraint:** During direct streaming, redirects are manually validated to remain on the exact same host or a legitimate dot-boundary subdomain. Foreign host hops fail immediately.
4. **Polite networking:** Downloads and resolves are spaced sequentially (controlled by `download-grace-seconds`) to prevent cloud host rate-limiting.

---

## 📦 Multi-Format Archive Extraction with 7-Zip CLI

Downloaded games are packed in diverse archive formats (`.zip`, `.7z`, `.rar`, `.tar.gz`, `.tar.bz2`, `.tar.xz`, and SFX `.exe`). To maximize decompression speed and ensure universal format compatibility, LewdZone Launcher utilizes the **7-Zip console executable** (`7za` / `7z` / `7zz`).

### Why 7-Zip CLI?
- **Speed & Multi-threading:** Decompresses multi-gigabyte archives substantially faster than native single-threaded extractors.
- **Universal Archive Support:** Seamlessly unpacks `.zip`, `.7z`, `.rar` (including RAR5), `.tar.gz`, `.tar.bz2`, `.tar.xz`, and multi-part archives.
- **Real-Time Progress:** Streams decompression progress percentage (`-bsp1`) back to the launcher UI in real time.
- **Clean Background Execution:** Spawns completely hidden (`CREATE_NO_WINDOW` on Windows) and cancels immediately if requested.

### 📥 7-Zip CLI Download Links (v26.03)

The 7-Zip command-line tool is available for all major platforms. You can download the latest version directly:

| Platform | Architecture | Official Direct Download |
| --- | --- | --- |
| **Windows** | x86 / x64 | [7z2603-extra.7z](https://github.com/ip7z/7zip/releases/download/26.03/7z2603-extra.7z) |
| **Linux** | 64-bit x86-64 | [7z2603-linux-x64.tar.xz](https://github.com/ip7z/7zip/releases/download/26.03/7z2603-linux-x64.tar.xz) |
| **Linux** | 32-bit x86 | [7z2603-linux-x86.tar.xz](https://github.com/ip7z/7zip/releases/download/26.03/7z2603-linux-x86.tar.xz) |
| **Linux** | 64-bit arm64 | [7z2603-linux-arm64.tar.xz](https://github.com/ip7z/7zip/releases/download/26.03/7z2603-linux-arm64.tar.xz) |
| **Linux** | 32-bit arm | [7z2603-linux-arm.tar.xz](https://github.com/ip7z/7zip/releases/download/26.03/7z2603-linux-arm.tar.xz) |
| **macOS** | arm64 / x86-64 | [7z2603-mac.tar.xz](https://github.com/ip7z/7zip/releases/download/26.03/7z2603-mac.tar.xz) |

General download directory and release notes are available on the [7-Zip Official Download Page](https://www.7-zip.org/download.html).

---

## 🛠️ Setting Up 7-Zip CLI

### 1. Windows Setup
1. Download [7z2603-extra.7z](https://github.com/ip7z/7zip/releases/download/26.03/7z2603-extra.7z).
2. Extract the archive into a permanent folder (for example, `C:\Utilities\7z` or `D:\Tools\7z`). Inside, you will find `7za.exe` (and `x64\7za.exe`).
3. In LewdZone Launcher, open **Settings**.
4. In the **7-Zip console executable path** field, enter either the directory (`C:\Utilities\7z`) or the direct path to the executable (`C:\Utilities\7z\7za.exe`).
5. *Auto-Detection:* If you place `7za.exe` in `C:\Utilities\7z` or have standard 7-Zip installed in `C:\Program Files\7-Zip`, the launcher will automatically detect it!

### 2. Linux Setup
- **Option A (Standalone):** Download the appropriate `.tar.xz` package for your CPU architecture, unpack it (e.g. `tar -xf 7z2603-linux-x64.tar.xz -C ~/.local/bin`), and ensure `7zz` is executable (`chmod +x ~/.local/bin/7zz`).
- **Option B (Package Manager):**
  - Debian/Ubuntu: `sudo apt install p7zip-full`
  - Arch Linux: `sudo pacman -S p7zip`
  - Fedora: `sudo dnf install p7zip p7zip-plugins`
- In LewdZone Settings, set `7z-path` to the binary path or rely on PATH detection (`7za`, `7z`, or `7zz`).

### 3. macOS Setup
- **Option A (Homebrew):** Run `brew install sevenzip`.
- **Option B (Standalone):** Download [7z2603-mac.tar.xz](https://github.com/ip7z/7zip/releases/download/26.03/7z2603-mac.tar.xz), extract `7zz`, and place it in `/usr/local/bin` or `~/bin`.
- In Settings, set `7z-path` to `/opt/homebrew/bin/7zz` (Apple Silicon) or `/usr/local/bin/7zz`.

### Extraction Fallback Hierarchy
If 7-Zip is not configured or fails, the launcher falls back gracefully:
1. **7-Zip CLI (`7za`/`7z`/`7zz`):** First priority for all formats.
2. **Native Zip Extractor:** In-memory streaming zip reader for standard `.zip` files.
3. **System `tar` (bsdtar):** Built into Windows, macOS, and Linux for zip, tar, and gzip.

---

## 📂 Library & Downloads Folder Layout

The library structure is organized flat to make browsing and backup simple:

```
<library-root>/
├── installed/
│   ├── game-slug/
│   │   ├── app.json                  # itch.io-style launcher manifest
│   │   ├── Game.exe                  # executable
│   │   └── game_data/
│   └── another-slug/
│       ├── app.json
│       └── Launch.exe
└── games/                            # optional custom games directory
    └── ...
<download-dir>/
├── Game Title [Ongoing] - Version 1.0.zip
└── Another Game - Version 0.5.rar
```

- **Downloads Directory:** By default the launcher uses the OS downloads folder
  (`%USERPROFILE%\Downloads`, `~/Downloads`, etc.). Configure a custom path in
  Settings with `download-dir`.
- **Installed Directory (`<library-root>/installed/<slug>/`):** Each extracted game resides in its own slug folder. An `app.json` manifest records the game metadata, engine, install path, launch executable candidates, and custom overrides.
- **Backwards Compatibility:** The launcher continues to recognize existing games located in legacy `<library-root>/lzapps/<slug>/` folders.
- **Custom Games Directory (`games-dir`):** If you store games in another custom directory (e.g. `D:\Games\LewdZone`), you can configure this in Settings and use the **Scan Games** button in the Library to automatically register them.

---

## 🧵 Download Queue Management

The **Downloads** page exposes full queue control:

- **Live Statuses:**
  - `queued`: Waiting for scheduler turn.
  - `resolving`: Contacting LewdZone API for download tokens.
  - `dispatching`: Routing to in-app stream or OS default handler.
  - `downloading`: Direct file streaming with byte progress and speed calculation.
  - `extracting`: Archive extraction with real-time percentage and byte progress from 7-Zip.
  - `completed`: Successfully extracted, registered in Library, and ready to play.
  - `dispatched`: Dispatched to OS browser or desktop cloud client.
  - `failed`: Errored with a descriptive message.
  - `cancelled`: Terminated by user.
- **Action Controls:**
  - **Cancel:** Instantly terminates active network downloads or running 7-Zip extraction child processes, preventing orphaned files.
  - **Delete:** Removes any finished, failed, or cancelled job from the queue display and SQLite database.
  - **Clear Finished:** Removes all completed jobs in a single click.

---

## 🔗 Related Pages

- [Configuration](Configuration) — `library-root`, `games-dir`, `7z-path`, and `source-priority`
- [CLI Reference](CLI-Reference) — `lewdzone download`, `lewdzone list --jobs`
- [Architecture](Architecture) — Two entry points and core dispatch flow
- [Troubleshooting](Troubleshooting) — Resolving extraction or 7-Zip path issues