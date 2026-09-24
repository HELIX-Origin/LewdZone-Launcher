# Privacy Policy for LewdZone-Launcher

**Last Updated:** September 21, 2026

This privacy policy explains how the LewdZone-Launcher project handles
information. LewdZone-Launcher is a Tauri 2 desktop application with a native
CLI that helps you browse and download games from **lewdzone.com**.

## 1. Core Principle: Self-Hosted & Zero Telemetry

**This project is self-hosted and collects zero telemetry.**

- No tracking pixels, analytics SDKs, crash reporters, or telemetry of any
  kind are embedded in the application.
- No central server operated by the project receives your data.
- All runtime data is stored **locally on your machine**. The project team
  cannot see, query, or exfiltrate it.

## 2. Information Handled by the Service

The application stores information **locally, on your device**, inside a
SQLite database that you own:

- **Catalog records:** game entries, titles, genres, versions, and download
  tokens as published by lewdzone.com.
- **Content metadata:** enrichment data (descriptions, cover art, ratings,
  screenshots) fetched from third-party content providers you configure.
- **Configuration:** your settings (download folder location, active download
  manager, provider API keys you paste in yourself).

API keys you enter (e.g. SteamGridDB, IGDB, VNDB) are stored in your local
configuration only and are used solely to authenticate with those providers on
your behalf. They are never transmitted to the project team.

## 3. External Network Interactions

The application makes outbound requests **only** when you use it to:

- Browse and scrape game pages from lewdzone.com.
- Resolve download tokens to real download URLs at dispatch time.
- Fetch enrichment data from content providers you have configured
  (SteamGridDB, VNDB, IGDB, itch.io, Steam, IndieDB).

No request is made in the background, on launch, or without a corresponding
user action.

## 4. Data Retention & Erasure

- Your data lives in your local database.
- **Deleting the local database deletes all application data** — including
  catalog records, tokens, configuration, and provider caches.
- Direct-file downloads stream in-app to your local `downloads/` staging folder
  and are extracted into `lzapps/`; other resolved URLs are opened by your OS
  default handler (installed cloud app or browser). The files are yours and are
  managed by you, not by this project.

## 5. Security Architecture

- No secrets are committed to the repository; API keys are kept in the SQLite
  `secret` table and are never written to the readable JSON config.
- Subprocess calls use argument arrays (never shell interpolation) to avoid
  injection (see [Security Policy](SECURITY.md)).
- The application never uploads your database or tokens anywhere.

## 6. Community & Contact

- GitHub: [HELIX-Origin/LewdZone-Launcher](https://github.com/HELIX-Origin/LewdZone-Launcher)
- Security: see [Security Policy](SECURITY.md) for responsible disclosure.

---

*This project is not affiliated with lewdzone.com or any content provider.
Trademarks and content belong to their respective owners.*