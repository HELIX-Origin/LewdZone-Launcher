# ADR-0006: Removal of external source providers and desktop shortcuts

- **Status:** accepted
- **Date:** 2026-09-23
- **Owner:** architect, content, shortcuts
- **Applies to:** Rule 00, Rule 03, Rule 05, Rule 13

## Context

1. **External Source Providers:** The content-provider enrichment layer (SteamGridDB, IGDB, VNDB, itch.io, Steam, IndieDB) introduced in ADR-0004 was intended to fill metadata gaps. In practice, external provider requests are unreliable, fail or hang, and directly block store game detail pages from loading. Furthermore, LewdZone's own scraped data already provides accurate downloads, descriptions, versions, tags, and screenshots.
2. **Desktop Shortcuts:** LewdZone pages do not provide square, high-resolution app icon assets for many games. Without external providers supplying icon artwork, desktop shortcuts generated on Windows, macOS, or Linux look broken or have missing icons. The desktop shortcut feature is therefore abandoned in favor of in-app library launching.
3. **Preferred Sources Configuration:** The `source-priority` setting was previously edited as an arbitrary comma-separated string, requiring users to manually know and type host slugs. A selection box / toggle UI is required for better usability.

## Decision

- **Remove External Providers:** Remove calls to `content_enrich` during game page loading. The store page transitions to `ready` immediately upon loading native LewdZone scraped game data. Disable external provider network queries in the Rust core.
- **Settings Cleanup:** Remove `content-priority` and API keys (SteamGridDB, IGDB) from the Settings UI.
- **Preferred Sources Toggles:** Replace the text input for `source-priority` with an interactive toggle interface exposing known, allowlisted host providers with friendly display names.
- **Abandon Desktop Shortcuts:** Remove the "Shortcut" button from the Library view. Mark shortcut creation in the backend/CLI as discontinued.

## Consequences

- Game detail pages load instantly with 0 external network dependencies.
- Library UI is cleaner and focused on in-app management and launch.
- Settings UI provides clear, error-free source priority toggling.
