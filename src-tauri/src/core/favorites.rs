//! Favorites persistence — heart-tab games backed by SQLite (owner: database
//! family, Rule 06 / GUI-CLI parity Rule 13).
//!
//! All operations are slug-driven on the surface, but stored by stable
//! `post_id` foreign key so renames do not break the user's list.

use crate::core::{models::GameCard, Context, Error};
use crate::db::{migrate, open};

/// Every favorited game as a `GameCard`, newest first.
pub fn list(ctx: &Context) -> Result<Vec<GameCard>, Error> {
    let conn = open(&ctx.db_path)?;
    migrate(&conn)?;
    crate::db::repo::favorite_list(&conn)
}

/// Add a game to favorites by slug. Errors if the slug is not in the catalog.
pub fn add(ctx: &Context, slug: &str) -> Result<(), Error> {
    let conn = open(&ctx.db_path)?;
    migrate(&conn)?;
    let post_id = crate::db::repo::post_id_by_slug(&conn, slug)?
        .ok_or_else(|| Error::Usage(format!("unknown game '{slug}'")))?;
    let tx = conn.unchecked_transaction()?;
    crate::db::repo::favorite_add(&tx, post_id)?;
    tx.commit()?;
    Ok(())
}

/// Remove a game from favorites by slug. Missing favorites are silently ignored.
pub fn remove(ctx: &Context, slug: &str) -> Result<(), Error> {
    let conn = open(&ctx.db_path)?;
    migrate(&conn)?;
    let post_id = crate::db::repo::post_id_by_slug(&conn, slug)?
        .ok_or_else(|| Error::Usage(format!("unknown game '{slug}'")))?;
    let tx = conn.unchecked_transaction()?;
    crate::db::repo::favorite_remove(&tx, post_id)?;
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::Game;
    use crate::db::migrate;
    use rusqlite::Connection;

    fn tmp_ctx(tag: &str) -> Context {
        let pid = std::process::id();
        let dir = std::env::temp_dir().join(format!("lz-fav-{tag}-{pid}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Context::new(dir.join("lewdzone.db"), dir.join("config.json"))
    }

    fn sample_game() -> Game {
        Game {
            slug: "wild-life".to_string(),
            post_id: Some(101),
            title: "Wild Life".to_string(),
            developer: Some("Adeptus Steve".to_string()),
            current_version: Some("v2026".to_string()),
            engine: Some("Unreal Engine".to_string()),
            platforms: vec!["pc".to_string()],
            genres: vec!["3d-game".to_string()],
            size_label: None,
            censorship: None,
            screenshots: vec![],
            description: Some("A mad universe".to_string()),
            versions: vec![],
            download_entries: vec![],
        }
    }

    #[test]
    fn favorites_roundtrip_by_slug() {
        let ctx = tmp_ctx("roundtrip");

        let conn = Connection::open(&ctx.db_path).unwrap();
        migrate(&conn).unwrap();
        let tx = conn.unchecked_transaction().unwrap();
        crate::db::repo::upsert_game(&tx, &sample_game(), None, None).unwrap();
        tx.commit().unwrap();
        drop(conn);

        assert!(list(&ctx).unwrap().is_empty());

        add(&ctx, "wild-life").unwrap();
        let list1 = list(&ctx).unwrap();
        assert_eq!(list1.len(), 1);
        assert_eq!(list1[0].slug, "wild-life");
        assert_eq!(list1[0].title, "Wild Life");

        remove(&ctx, "wild-life").unwrap();
        assert!(list(&ctx).unwrap().is_empty());
    }

    #[test]
    fn favorites_add_rejects_unknown_slug() {
        let ctx = tmp_ctx("unknown");
        let err = add(&ctx, "does-not-exist").unwrap_err().to_string();
        assert!(err.contains("does-not-exist"));
    }
}
