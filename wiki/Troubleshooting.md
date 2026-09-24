# 🐛 Troubleshooting

> Links between wiki pages are relative and omit the `.md` extension.

This guide covers common issues and questions regarding downloads, extraction, 7-Zip configuration, and the system tray.

---

## 📦 7-Zip CLI Not Found or Extraction Errors

### Symptom
- Extraction fails with a message indicating 7-Zip could not be found or executed.
- Non-zip formats (such as `.7z` or `.rar`) do not decompress automatically.

### Resolution
1. **Download the 7-Zip Standalone CLI:** Download the console package for your OS from the **[Archive Extraction & 7-Zip Setup Guide](Archive-Extraction)** (e.g. `7z2603-extra.7z` for Windows or `.tar.xz` for Linux/macOS).
2. **Extract to a Permanent Directory:** Unpack the contents into a stable location, such as:
   ```text
   C:\Utilities\7z\
   ```
   *(Ensure `7za.exe` is inside this directory).*
3. **Configure the Path in Settings:**
   - Go to **Settings** (`/settings`) in the launcher.
   - Enter `C:\Utilities\7z` or `C:\Utilities\7z\7za.exe` into the **7-Zip console executable path** field.
   - Alternatively via CLI: `lewdzone settings set 7z-path "C:\Utilities\7z\7za.exe"`.
4. **Auto-Detection:** LewdZone Launcher automatically inspects `C:\Utilities\7z`, `C:\Program Files\7-Zip\7z.exe`, and your system `PATH`. Placing `7za.exe` in `C:\Utilities\7z` allows it to be detected automatically!

---

## ⚠️ Cloud-Host Downloads Open in Browser Instead of In-App

### Explanation
- **Direct-file hosts** (`fileknot`) are streamed directly inside the app with byte counters and progress metrics.
- **Cloud-file hosts** (such as MEGA, Google Drive, Dropbox, MediaFire, Pixeldrain, WorkUpload) intentionally hand the resolved URL to your **OS default handler** (installed desktop cloud app or default browser).
- This is designed behavior to respect host-specific authentication, download limits, and multi-tier rate limiting.

---

## ↩️ Verification Countdown or Captcha Prompt

### Explanation
- Some download hosts require a countdown timer or interactive human verification.
- When this occurs, LewdZone Launcher opens its **in-app sandboxed webview resolver window**.
- This window is strictly isolated from third-party advertising, malicious redirects, and popups. Complete the on-screen prompt, and the launcher will automatically capture the verified download link and resume.

---

## 🛎️ Application Closes to System Tray

### Explanation
- Closing or minimizing the main window docks LewdZone Launcher into your system tray so active downloads and extractions continue uninterrupted in the background.
- **To reopen:** Click or double-click the LewdZone tray icon, or right-click the tray icon and choose **Open LewdZone Launcher**.
- **To exit completely:** Right-click the tray icon and select **Quit LewdZone**, or choose **File > Quit** from the application menu.

---

## 🎮 Games Not Appearing in Library

### Resolution
1. **Verify Library Root:** Ensure `library-root` in **Settings** points to your intended library directory (e.g. `G:\LewdZone`). Installed games reside in `<library-root>/installed/<slug>/app.json`.
2. **Scan Custom Games Directory:** If you personally extract games into a custom directory, set **Extracted games directory** (`games-dir`) in **Settings** (e.g. `D:\Games\LewdZone`), navigate to the **Library**, and click **Scan Games**. The launcher will automatically inspect the directory, identify executables, generate manifests, and register your games.

---

## 🐌 Slow Catalog Synchronization

### Explanation
- LewdZone Launcher enforces polite network etiquette (~1 request per second) to prevent overloading the server or triggering IP rate-limits ([Security](Security)).
- The initial sync takes a few moments to page through the catalog. Subsequent syncs are incremental and only fetch new or updated pages.

---

## 🧹 Corrupted or Stuck Download Jobs

### Resolution
1. Navigate to the **Downloads** view.
2. Click **Cancel** on any active or stalled download.
3. Click the **Delete** (trash) icon on any completed, failed, or cancelled jobs to prune them from the queue.
4. Click **Clear Finished** to purge all finished items in one action.

---

## 🔗 Related Pages

- [Archive Extraction & 7-Zip Guide](Archive-Extraction)
- [Downloads & In-App Streaming](Download-Managers)
- [Configuration](Configuration)
- [CLI Reference](CLI-Reference)