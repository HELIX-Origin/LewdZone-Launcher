# 🐛 BUGS (open issues)

Only open bugs belong here. Each entry links to its GitHub issue (once filed)
and is closed by editing this file, not by deleting history. When a bug is fixed,
move it to the commit that resolved it (`git log`).

## 2026-09-23 — Downloads fail redirect challenge & source selection needed
- **Status:** Open
- **Description:**
  - Downloads currently do not work because automated requests fail to get past the site's redirect challenge on their own.
  - A proper child webview window setup needs to be implemented to allow users to interact with and click the final download button so the launcher app can pick up the resolved download link directly.
  - Selecting the download source directly from the game's store page needs to be supported.
- **Steps to reproduce:**
  1. Open the Store page and select any game (e.g. `treasure-of-nadia`).
  2. Attempt to resolve tokens or queue a download through the background queue.
  3. Automated requests fail to bypass the host redirect challenge.
- **Planned fix:**
  - Launch a child webview window to allow completing any human verification / redirect clicks, and capture the final direct file or host URL from navigation/download events.
  - Enable explicit download source selection directly on the store game detail page.

## ✅ Resolved

### 2026-09-23 — Store game pages failing to load & hanging on external providers
- **Root cause:**
  1. External metadata providers (SteamGridDB, VNDB, IGDB, itch, Steam, IndieDB) were hanging on network calls and delaying page renders.
  2. Svelte view threw a TypeError on `external_genres.length`.
  3. Single-version games without dropdowns lost their download entries.
  4. Redundant rate-limited network fetches occurred because data was not cached in memory or loaded from the database.
- **Fix:** Removed external source providers (ADR-0006); synthesized default version for single-version games; implemented memory-first caching in `AppState` (`game_cache`, `page_cache`, `genres_cache`) backed by SQLite persistence; full metadata and screenshot deserialization in `db::repo::game_by_slug`; deduplicated load triggers in `store/[slug]/+page.svelte`.
- **Resolved in:** commit `c7a19eb`.

### 2026-09-23 — `core::favorites` roundtrip test fails on Windows
- **Root cause:** The test helper removed the temp dir but never recreated it, and `favorite_list` selected a non-existent `game.views` column.
- **Fix:** `std::fs::create_dir_all` in `core/favorites.rs` test helper; removed `g.views` from `favorite_list` SQL and set `GameCard.views` to `None`.
- **Resolved in:** commit `a08ca6a`.

## 📝 Filing a bug

Bug title on GitHub: `🐛 <problem summary>`. Body must include:

- Steps to reproduce (reproduce-first)
- Expected vs actual behavior
- Environment (OS, app/CLI version, DM in use)
- ≥1 verifiable diagram or log when applicable ([Rule 04](.agents/rules/rule-04-remote-issue-protocol.md))

Entry format once filed:

```
## 2026-09-21 — <short title>  (#<issue>)
- [ ] Reproduced
- [ ] Root cause identified
- [ ] Fix in PR (`Closes #<issue>`)
```