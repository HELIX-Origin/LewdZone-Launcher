# 📦 Archive Extraction & 7-Zip Setup Guide

> Links between wiki pages are relative and omit the `.md` extension.

LewdZone Launcher uses the **7-Zip console executable** (`7za`, `7z`, or `7zz`) to power fast, multi-threaded extraction of all supported archive formats with live percentage progress tracking.

---

## 🚀 Why 7-Zip CLI?

Game releases on LewdZone are distributed in many archive formats, including large multi-gigabyte `.zip`, `.7z`, `.rar`, `.tar.gz`, `.tar.bz2`, `.tar.xz`, and self-extracting `.exe` (SFX) archives. Using the standalone 7-Zip command-line tool provides:

1. **High Decompression Performance:** Up to 5-10× faster extraction speeds compared to basic zip readers, utilizing all CPU cores.
2. **Universal Format Support:** Full decompression support for `.zip`, `.7z`, `.rar` (including RAR5), `.tar.gz`, `.tar.bz2`, `.tar.xz`, and `.exe` SFX.
3. **Real-Time Progress Streaming:** Outputs interactive percentage benchmarks (`-bsp1`), allowing the launcher to calculate byte-accurate extraction progress in real time.
4. **No Full Installation Required:** The standalone console version is a portable binary that requires no system installer, registry changes, or administrator privileges.

---

## 📥 Direct Download Links (7-Zip v26.03)

Download the standalone console archive for your platform directly:

| Operating System | Target CPU / Format | Direct Download Link | Executable Inside |
| --- | --- | --- | --- |
| **Windows** | x86 / x64 (`.7z`) | [7z2603-extra.7z](https://github.com/ip7z/7zip/releases/download/26.03/7z2603-extra.7z) | `7za.exe` / `x64\7za.exe` |
| **Linux** | 64-bit x86-64 (`.tar.xz`) | [7z2603-linux-x64.tar.xz](https://github.com/ip7z/7zip/releases/download/26.03/7z2603-linux-x64.tar.xz) | `7zz` |
| **Linux** | 32-bit x86 (`.tar.xz`) | [7z2603-linux-x86.tar.xz](https://github.com/ip7z/7zip/releases/download/26.03/7z2603-linux-x86.tar.xz) | `7zz` |
| **Linux** | 64-bit arm64 (`.tar.xz`) | [7z2603-linux-arm64.tar.xz](https://github.com/ip7z/7zip/releases/download/26.03/7z2603-linux-arm64.tar.xz) | `7zz` |
| **Linux** | 32-bit arm (`.tar.xz`) | [7z2603-linux-arm.tar.xz](https://github.com/ip7z/7zip/releases/download/26.03/7z2603-linux-arm.tar.xz) | `7zz` |
| **macOS** | Universal arm64 / x86-64 (`.tar.xz`) | [7z2603-mac.tar.xz](https://github.com/ip7z/7zip/releases/download/26.03/7z2603-mac.tar.xz) | `7zz` |

You can also check the [7-Zip Official Download Page](https://www.7-zip.org/download.html) for newer updates and source releases.

---

## 🛠️ Step-by-Step Installation & Configuration

### Windows

1. Download **[7z2603-extra.7z](https://github.com/ip7z/7zip/releases/download/26.03/7z2603-extra.7z)**.
2. Extract the archive into a dedicated utilities or portable apps folder, e.g.:
   ```text
   C:\Utilities\7z\
   ```
    *(Ensure `7za.exe` and `x64\7za.exe` are inside that directory; `7z.exe` from a standard 7-Zip install is also accepted).*
3. Open **LewdZone Launcher** and navigate to the **Settings** page.
4. Locate the **7-Zip console executable path** setting.
5. Enter either the directory or the executable path:
   ```text
   C:\Utilities\7z
   ```
   *or*
   ```text
   C:\Utilities\7z\7za.exe
   ```
6. **Automatic Detection:** The launcher automatically checks `C:\Utilities\7z`, standard `C:\Program Files\7-Zip\7z.exe`, and your system `PATH`. If you place it at `C:\Utilities\7z`, it will be recognized even without setting the path!

### Linux

1. Download the archive matching your architecture (e.g. `7z2603-linux-x64.tar.xz`).
2. Extract the standalone binary `7zz` into a folder on your path:
   ```bash
   tar -xf 7z2603-linux-x64.tar.xz -C ~/.local/bin 7zz
   chmod +x ~/.local/bin/7zz
   ```
3. Alternatively, install via your package manager:
   - Ubuntu/Debian: `sudo apt install p7zip-full`
   - Arch Linux: `sudo pacman -S p7zip`
   - Fedora: `sudo dnf install p7zip p7zip-plugins`
4. Set the path in Settings or CLI if not in standard PATH:
   ```bash
   lewdzone settings set 7z-path ~/.local/bin/7zz
   ```

### macOS

1. **Via Homebrew (Recommended):**
   ```bash
   brew install sevenzip
   ```
2. **Via Standalone Download:**
   Download `7z2603-mac.tar.xz`, extract `7zz`, and place it in `/usr/local/bin` or `/opt/homebrew/bin`:
   ```bash
   tar -xf 7z2603-mac.tar.xz 7zz
   chmod +x 7zz
   sudo mv 7zz /usr/local/bin/
   ```
3. In the launcher Settings, configure:
   ```text
   /opt/homebrew/bin/7zz
   ```

---

## ⚙️ Configuration Reference

You can configure 7-Zip via the GUI **Settings** view or directly via the command-line interface:

```bash
# Set custom 7-Zip path
lewdzone settings set 7z-path "C:\Utilities\7z\7za.exe"

# Retrieve current 7-Zip path setting
lewdzone settings get 7z-path
```

Configuration aliases recognized:
- `7z-path`
- `seven-zip-path`

---

## 🔄 Automatic Fallback Hierarchy

If 7-Zip CLI is not detected or fails to process a file, the launcher reports the failure and the archive remains in the configured download directory. The background scanner will later retry extraction once 7-Zip is configured.

---

## 🔗 Related Pages

- [Download Managers & Streaming](Download-Managers) — Complete download workflow and lifecycle
- [Configuration](Configuration) — All settings and persistent keys
- [Troubleshooting](Troubleshooting) — Common extraction issues and solutions
