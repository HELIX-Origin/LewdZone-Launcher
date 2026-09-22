//! Repository functions — upserts, reads, sync_state (owner: database family).
//!
//! Identity rules (sync-orchestrator Rule 04 / Rule 06):
//! - `post_id` is the stable identity for a game; `slug` is UNIQUE and can be
//!   updated when the site renames a permalink.
//! - Download `token`s are stored, never resolved URLs (ADR-0003).
//! - Card metadata (`updated_at`, `thumbnail_url`) is only known to the
//!   archive card; the sync passes it in alongside the parsed Game page.

use std::collections::HashSet;

use rusqlite::{params, Connection, OptionalExtension};

use crate::core::models::{DownloadEntry, Game, Version};
use crate::core::Error;

fn token_of(go_link: &str) -> String {
    let trimmed = go_link.trim();
    match trimmed.split_once("#t=") {
        Some((_, token)) => token.trim().to_string(),
        None => trimmed.to_string(),
    }
}

/// Import a Game plus card metadata inside a caller-managed transaction.
/// Idempotent: repeated calls converge to the same rows (verified by tests).
pub fn upsert_game(
    tx: &Connection,
    game: &Game,
    card_updated_at: Option<&str>,
    card_thumbnail: Option<&str>,
) -> Result<(), Error> {
    // post_id is the stable identity; a card/game without one is not storable.
    let Some(post_id) = game.post_id else {
        return Ok(());
    };
    tx.execute(
        r#"
        INSERT INTO game (post_id, slug, title, developer, engine, size_label,
                          censorship, description, updated_at, thumbnail_url)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        ON CONFLICT(slug) DO UPDATE SET
            post_id = excluded.post_id,
            title = excluded.title,
            developer = excluded.developer,
            engine = excluded.engine,
            size_label = excluded.size_label,
            censorship = excluded.censorship,
            description = excluded.description,
            updated_at = excluded.updated_at,
            thumbnail_url = excluded.thumbnail_url
        "#,
        params![
            post_id,
            game.slug,
            game.title,
            game.developer,
            game.engine,
            game.size_label,
            game.censorship,
            game.description,
            card_updated_at,
            card_thumbnail,
        ],
    )?;

    // Genres: upsert genre rows, re-wire the join.
    tx.execute("DELETE FROM game_genre WHERE game_id = ?1", [post_id])?;
    for genre in &game.genres {
        tx.execute(
            "INSERT INTO genre (slug, label) VALUES (?1, ?1) ON CONFLICT(slug) DO NOTHING",
            [genre],
        )?;
        tx.execute(
            "INSERT OR IGNORE INTO game_genre (game_id, genre_id) VALUES (?1, ?2)",
            params![post_id, genre],
        )?;
    }

    // Versions → entries. Tab comes from the owning vector (official/community).
    for version in &game.versions {
        upsert_version(tx, post_id, version)?;
    }
    Ok(())
}

fn upsert_version(tx: &Connection, post_id: i64, version: &Version) -> Result<(), Error> {
    let label = version.label.clone();
    let is_latest = version.is_latest as i64;

    tx.execute(
        r#"
        INSERT INTO version (game_id, label, is_latest)
        VALUES (?1, ?2, ?3)
        ON CONFLICT(game_id, label) DO UPDATE SET is_latest = excluded.is_latest
        "#,
        params![post_id, label, is_latest],
    )?;

    let version_id: i64 = tx.query_row(
        "SELECT id FROM version WHERE game_id = ?1 AND label = ?2",
        params![post_id, label],
        |row| row.get(0),
    )?;

    // Replace entries of this version (idempotent: delete-then-insert).
    tx.execute(
        "DELETE FROM download_entry WHERE version_id = ?1",
        [version_id],
    )?;
    for entry in &version.official {
        upsert_entry(tx, post_id, version_id, "official", entry)?;
    }
    for entry in &version.community {
        upsert_entry(tx, post_id, version_id, "community", entry)?;
    }
    Ok(())
}

fn upsert_entry(
    tx: &Connection,
    post_id: i64,
    version_id: i64,
    tab: &str,
    entry: &DownloadEntry,
) -> Result<(), Error> {
    tx.execute(
        "INSERT INTO host (slug, display_name) VALUES (?1, ?1) ON CONFLICT(slug) DO NOTHING",
        [&entry.host],
    )?;
    tx.execute(
        r#"
        INSERT INTO download_entry (game_id, version_id, host_slug, platform, tab, label, variant, token)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ON CONFLICT(game_id, version_id, host_slug, platform, tab, label, variant) DO UPDATE SET
            token = excluded.token
        "#,
        params![
            post_id,
            version_id,
            entry.host,
            entry.platform.clone().unwrap_or_default(),
            tab,
            entry.label,
            entry.variant,
            token_of(&entry.go_link),
        ],
    )?;
    Ok(())
}

/// Card `updated_at` for change detection during incremental sync.
pub fn game_updated_at(tx: &Connection, slug: &str) -> Result<Option<String>, Error> {
    tx.query_row(
        "SELECT updated_at FROM game WHERE slug = ?1",
        [slug],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .map_err(Into::into)
}

/// Full game with genres, versions, and entries (tokens → go_links) for
/// round-trip verification.
pub fn game_by_slug(tx: &Connection, slug: &str) -> Result<Option<Game>, Error> {
    let post_id: Option<i64> = tx
        .query_row(
            r#"
            SELECT post_id FROM game WHERE slug = ?1
            "#,
            [slug],
            |row| row.get(0),
        )
        .optional()?;
    let Some(post_id) = post_id else {
        return Ok(None);
    };

    let title: String = tx.query_row(
        "SELECT title FROM game WHERE post_id = ?1",
        [post_id],
        |row| row.get(0),
    )?;

    let genres: Vec<String> = {
        let mut stmt = tx.prepare(
            "SELECT genre.slug FROM game_genre JOIN genre ON genre.slug = game_genre.genre_id
             WHERE game_genre.game_id = ?1 ORDER BY genre.label",
        )?;
        let rows = stmt.query_map([post_id], |row| row.get(0))?;
        let mut out = Vec::new();
        for g in rows {
            out.push(g?);
        }
        out
    };

    let versions: Vec<Version> = {
        let mut stmt = tx.prepare(
            "SELECT id, label, is_latest FROM version WHERE game_id = ?1
             ORDER BY is_latest DESC, label DESC",
        )?;
        let rows = stmt.query_map([post_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?;
        let mut out = Vec::new();
        for v in rows {
            let (vid, label, is_latest) = v?;
            let (official, community) = load_entries(tx, vid)?;
            out.push(Version {
                label,
                is_latest: is_latest != 0,
                official,
                community,
            });
        }
        out
    };

    // Single flat entry view; DB does not preserve the model's
    // `download_entries` ordering, so rebuild from versions.
    let mut download_entries: Vec<DownloadEntry> = Vec::new();
    for v in &versions {
        download_entries.extend(v.official.iter().cloned());
        download_entries.extend(v.community.iter().cloned());
    }

    let model = Game {
        slug: slug.to_string(),
        post_id: Some(post_id),
        title,
        developer: None,
        current_version: versions
            .iter()
            .find(|v| v.is_latest)
            .map(|v| v.label.clone()),
        engine: None,
        platforms: Vec::new(),
        genres,
        size_label: None,
        censorship: None,
        screenshots: Vec::new(),
        description: None,
        versions,
        download_entries,
    };
    Ok(Some(model))
}

fn load_entries(
    tx: &Connection,
    version_id: i64,
) -> Result<(Vec<DownloadEntry>, Vec<DownloadEntry>), Error> {
    let mut stmt = tx.prepare(
        "SELECT host_slug, platform, tab, label, variant, token
         FROM download_entry WHERE version_id = ?1 ORDER BY id",
    )?;
    let rows = stmt.query_map([version_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, Option<String>>(4)?,
            row.get::<_, String>(5)?,
        ))
    })?;
    let mut official = Vec::new();
    let mut community = Vec::new();
    for e in rows {
        let (host, platform, tab, label, variant, token) = e?;
        let entry = DownloadEntry {
            label: label.unwrap_or_default(),
            variant,
            host,
            platform: if platform.is_empty() {
                None
            } else {
                Some(platform)
            },
            go_link: format!("https://lewdzone.com/go/#t={token}"),
        };
        if tab == "official" {
            official.push(entry);
        } else {
            community.push(entry);
        }
    }
    Ok((official, community))
}

/// sync_state helpers (resumable sync, ADR-0003).
pub fn sync_state_get(tx: &Connection, key: &str) -> Result<Option<String>, Error> {
    tx.query_row(
        "SELECT value FROM sync_state WHERE key = ?1",
        [key],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .map_err(Into::into)
}

pub fn sync_state_set(tx: &Connection, key: &str, value: &str) -> Result<(), Error> {
    tx.execute(
        "INSERT INTO sync_state (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

/// Prune games whose slug no longer appears in a fresh full sync, returning
/// the number removed. Only run after a clean full sync (sync-orchestrator 6).
/// Parameterized placeholders — slugs are never string-interpolated.
pub fn prune_slugs(tx: &Connection, seen: &HashSet<String>) -> Result<u32, Error> {
    let mut slugs: Vec<&str> = seen.iter().map(String::as_str).collect();
    slugs.sort_unstable();
    if slugs.is_empty() {
        return Ok(0);
    }
    let placeholders = vec!["?"; slugs.len()].join(",");
    let sql = format!("DELETE FROM game WHERE slug NOT IN ({placeholders})");
    let mut stmt = tx.prepare(&sql)?;
    let removed = stmt.execute(rusqlite::params_from_iter(slugs.iter()))?;
    Ok(removed as u32)
}

/// Count games (test utility).
pub fn game_count(tx: &Connection) -> Result<i64, Error> {
    tx.query_row("SELECT COUNT(*) FROM game", [], |row| row.get(0))
        .map_err(Into::into)
}

/// Resolve a stable post id to its current permalink slug (inverse of the
/// slug UNIQUE index). Used by `info`/`download` when the user passes an id.
pub fn slug_by_post_id(tx: &Connection, post_id: i64) -> Result<Option<String>, Error> {
    tx.query_row(
        "SELECT slug FROM game WHERE post_id = ?1",
        [post_id],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .map_err(Into::into)
}

/// Compact catalog rows for `list` / Storefront tiles: each game with its
/// current version label and genre slugs, ordered by title.
#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct CatalogGame {
    pub post_id: i64,
    pub slug: String,
    pub title: String,
    pub developer: Option<String>,
    pub size_label: Option<String>,
    pub censorship: Option<String>,
    pub updated_at: Option<String>,
    pub current_version: Option<String>,
    pub genres: Vec<String>,
}

pub fn list_catalog(tx: &Connection, limit: Option<i64>) -> Result<Vec<CatalogGame>, Error> {
    let mut stmt = tx.prepare(
        r#"
        SELECT g.post_id, g.slug, g.title, g.developer, g.size_label,
               g.censorship, g.updated_at,
               (SELECT v.label FROM version v
                 WHERE v.game_id = g.post_id AND v.is_latest = 1
                 ORDER BY v.label DESC LIMIT 1)
        FROM game g
        ORDER BY g.title COLLATE NOCASE
        "#,
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, Option<String>>(3)?,
            r.get::<_, Option<String>>(4)?,
            r.get::<_, Option<String>>(5)?,
            r.get::<_, Option<String>>(6)?,
            r.get::<_, Option<String>>(7)?,
        ))
    })?;
    let mut out = Vec::new();
    for row in rows {
        let (post_id, slug, title, developer, size_label, censorship, updated_at, current_version) =
            row?;
        let genres: Vec<String> = {
            let mut gs = tx.prepare(
                "SELECT genre.slug FROM game_genre JOIN genre ON genre.slug = game_genre.genre_id
                 WHERE game_genre.game_id = ?1 ORDER BY genre.label",
            )?;
            let gr = gs.query_map([post_id], |g| g.get::<_, String>(0))?;
            let mut v = Vec::new();
            for g in gr {
                v.push(g?);
            }
            v
        };
        out.push(CatalogGame {
            post_id,
            slug,
            title,
            developer,
            size_label,
            censorship,
            updated_at,
            current_version,
            genres,
        });
        if let Some(limit) = limit {
            if out.len() as i64 >= limit {
                break;
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrate;

    fn mem() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        conn
    }

    fn sample_game() -> Game {
        let official = DownloadEntry {
            label: "FULL GAME".to_string(),
            variant: None,
            host: "fileknot".to_string(),
            platform: Some("pc".to_string()),
            go_link: "https://lewdzone.com/go/#t=v1.raw".to_string(),
        };
        let community = DownloadEntry {
            label: "Part 1".to_string(),
            variant: Some("part-1".to_string()),
            host: "hx2go".to_string(),
            platform: Some("android".to_string()),
            go_link: "https://lewdzone.com/go/#t=v1.raw2".to_string(),
        };
        Game {
            slug: "treasure-of-nadia".to_string(),
            post_id: Some(18212),
            title: "Treasure of Nadia".to_string(),
            developer: Some("NLT Media".to_string()),
            current_version: Some("v1.0.0".to_string()),
            engine: None,
            platforms: vec!["pc".to_string(), "android".to_string()],
            genres: vec!["adventure".to_string(), "vore".to_string()],
            size_label: Some("7.51 GB".to_string()),
            censorship: None,
            screenshots: vec![],
            description: Some("a pirate arg game".to_string()),
            versions: vec![Version {
                label: "v1.0.0".to_string(),
                is_latest: true,
                official: vec![official.clone()],
                community: vec![community.clone()],
            }],
            download_entries: vec![official, community],
        }
    }

    #[test]
    fn upsert_roundtrips() {
        let conn = mem();
        let tx = conn.unchecked_transaction().unwrap();
        upsert_game(
            &tx,
            &sample_game(),
            Some("2024-10-01"),
            Some("https://cdn.example.com/t.jpg"),
        )
        .unwrap();
        tx.commit().unwrap();

        let got = game_by_slug(&conn, "treasure-of-nadia").unwrap().unwrap();
        assert_eq!(got.title, "Treasure of Nadia");
        assert_eq!(got.post_id, Some(18212));
        assert_eq!(
            got.genres,
            vec!["adventure".to_string(), "vore".to_string()]
        );
        assert_eq!(got.versions.len(), 1);
        assert_eq!(got.versions[0].label, "v1.0.0");
        assert_eq!(got.download_entries.len(), 2);
        let token = got.download_entries[0].go_link.split("#t=").nth(1).unwrap();
        assert_eq!(token, "v1.raw");
    }

    #[test]
    fn upsert_is_idempotent() {
        let conn = mem();
        for _ in 0..2 {
            let tx = conn.unchecked_transaction().unwrap();
            upsert_game(&tx, &sample_game(), Some("2024-10-01"), None).unwrap();
            tx.commit().unwrap();
        }
        assert_eq!(game_count(&conn).unwrap(), 1);
        let got = game_by_slug(&conn, "treasure-of-nadia").unwrap().unwrap();
        assert_eq!(got.download_entries.len(), 2);
    }

    #[test]
    fn game_updated_at_reflects_card() {
        let conn = mem();
        let tx = conn.unchecked_transaction().unwrap();
        upsert_game(&tx, &sample_game(), Some("2024-10-01"), None).unwrap();
        tx.commit().unwrap();
        assert_eq!(
            game_updated_at(&conn, "treasure-of-nadia")
                .unwrap()
                .as_deref(),
            Some("2024-10-01")
        );
    }

    #[test]
    fn upsert_updates_existing_slug() {
        let conn = mem();
        let mut a = sample_game();
        a.post_id = Some(1);
        a.slug = "game-a".to_string();
        let mut b = sample_game();
        b.post_id = Some(2);
        b.slug = "game-b".to_string();

        let tx = conn.unchecked_transaction().unwrap();
        upsert_game(&tx, &a, None, None).unwrap();
        upsert_game(&tx, &b, None, None).unwrap();
        tx.commit().unwrap();
        assert_eq!(game_count(&conn).unwrap(), 2);
    }

    #[test]
    fn slug_by_post_id_lookup() {
        let conn = mem();
        let tx = conn.unchecked_transaction().unwrap();
        upsert_game(&tx, &sample_game(), Some("2024-10-01"), None).unwrap();
        tx.commit().unwrap();
        assert_eq!(
            slug_by_post_id(&conn, 18212).unwrap().as_deref(),
            Some("treasure-of-nadia")
        );
        assert_eq!(slug_by_post_id(&conn, 999).unwrap(), None);
    }

    #[test]
    fn prune_removes_unseen_only() {
        let conn = mem();
        let mut a = sample_game();
        a.post_id = Some(1);
        a.slug = "game-a".to_string();
        let mut b = sample_game();
        b.post_id = Some(2);
        b.slug = "game-b".to_string();
        let tx = conn.unchecked_transaction().unwrap();
        upsert_game(&tx, &a, None, None).unwrap();
        upsert_game(&tx, &b, None, None).unwrap();
        tx.commit().unwrap();

        let tx = conn.unchecked_transaction().unwrap();
        let removed = prune_slugs(&tx, &HashSet::from(["game-a".to_string()])).unwrap();
        tx.commit().unwrap();

        assert_eq!(removed, 1);
        assert_eq!(game_count(&conn).unwrap(), 1);
        assert!(game_by_slug(&conn, "game-a").unwrap().is_some());
    }

    #[test]
    fn sync_state_roundtrips() {
        let conn = mem();
        let tx = conn.unchecked_transaction().unwrap();
        sync_state_set(&tx, "last_page", "7").unwrap();
        tx.commit().unwrap();
        assert_eq!(
            sync_state_get(&conn, "last_page").unwrap().as_deref(),
            Some("7")
        );
        let tx = conn.unchecked_transaction().unwrap();
        sync_state_set(&tx, "last_page", "8").unwrap();
        tx.commit().unwrap();
        assert_eq!(
            sync_state_get(&conn, "last_page").unwrap().as_deref(),
            Some("8")
        );
    }
}
