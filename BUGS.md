# 🐛 BUGS (open issues)

Only open bugs belong here. Each entry links to its GitHub issue (once filed)
and is closed by editing this file, not by deleting history. When a bug is fixed,
move it to the commit that resolved it (`git log`).

## ✅ Resolved

### 2026-09-23 — Downloads redirect challenge failure & invalid Zip archive (missing EOCD)
- **Root cause:**
  1. LewdZone go-links rely on a two-step `api.php` protocol with human countdowns. Direct requests or visiting external redirect URLs exposed users to aggressive third-party ads and popup injection.
  2. Cloud hosts returning 200 OK HTML landing pages were streamed into `.zip` files by direct downloaders; extraction then crashed with `invalid Zip archive: Could not find EOCD`.
  3. Game storefront lacked direct source selection and per-row download triggers.
- **Fix:**
  - Implemented custom in-app child resolver window (`/resolver`) with isolated ad-free countdown and token resolution flow.
  - Added `Content-Type` header inspection in `scraper::fetch::download_stream` to reject HTML payloads.
  - Added zip magic byte inspection (`PK\x03\x04`) in `core::extract::extract_zip` to surface clear errors on non-binary files.
  - Mimicked LewdZone's authentic per-source download layout on the store game page, using branded provider icons for all supported hosts.
  - Added queue cancellation, single-job deletion, and "Clear Finished" capabilities.
- **Resolved in:** commit pending.

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