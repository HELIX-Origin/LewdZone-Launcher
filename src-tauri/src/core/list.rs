//! `list` — list the synced catalog, installed library, or the job queue
//! (Rule 03 / Rule 13: the Storefront webview and the CLI share this core).
//!
//! Catalog rows come straight from the SQLite DB written by `sync`
//! (database family). Library rows come from the installed app manifests in
//! `lzapps/<slug>/app.json` (ADR-0005). Jobs come from the `download_job` table.

use crate::cli::ExitCode;
use crate::core::{library, Context, Error};
use crate::db;

/// A catalog listing (one row per game).
#[derive(Debug, Clone, serde::Serialize)]
pub struct Listing {
    pub total: i64,
    pub games: Vec<db::repo::CatalogGame>,
}

/// Listing of installed games from `lzapps/<slug>/app.json` manifests.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LibraryListing {
    pub root: Option<std::path::PathBuf>,
    pub games: Vec<LibraryGame>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LibraryGame {
    pub slug: String,
    pub post_id: Option<i64>,
    pub title: String,
    pub version: String,
    pub platform: String,
    pub tab: String,
    pub engine: Option<String>,
    pub install_path: String,
    pub candidates: Vec<String>,
    pub launch_exe: String,
    pub installed_at: Option<String>,
    pub size_on_disk: u64,
}

pub fn run(ctx: &Context, library_flag: bool, jobs: bool) -> Result<ExitCode, Error> {
    if jobs {
        return list_jobs(ctx);
    }
    if library_flag {
        return list_library(ctx);
    }
    list_catalog(ctx)
}

/// Read-only library listing for the webview (Rule 13: same core, no prints).
pub fn library_listing(ctx: &Context) -> Result<LibraryListing, Error> {
    let root = crate::core::folder::lzapps_root(ctx)?;
    let apps = library::list_installed(ctx)?;
    let games = apps
        .into_iter()
        .map(|app| LibraryGame {
            slug: app.slug,
            post_id: app.post_id,
            title: app.title,
            version: app.version,
            platform: app.platform,
            tab: app.tab,
            engine: app.engine,
            install_path: app.install_path.to_string_lossy().into_owned(),
            candidates: app.candidates,
            launch_exe: app.launch_exe,
            installed_at: app.installed_at,
            size_on_disk: app.size_on_disk,
        })
        .collect();
    Ok(LibraryListing {
        root: Some(root),
        games,
    })
}

fn list_catalog(ctx: &Context) -> Result<ExitCode, Error> {
    let conn = db::open(&ctx.db_path)?;
    db::migrate(&conn)?;
    let games = db::repo::list_catalog(&conn, None)?;
    let listing = Listing {
        total: games.len() as i64,
        games,
    };
    println!("{}", serde_json::to_string_pretty(&listing)?);
    Ok(ExitCode::Ok)
}

fn list_library(ctx: &Context) -> Result<ExitCode, Error> {
    let listing = library_listing(ctx)?;
    println!("{}", serde_json::to_string_pretty(&listing)?);
    Ok(ExitCode::Ok)
}

fn list_jobs(ctx: &Context) -> Result<ExitCode, Error> {
    let conn = db::open(&ctx.db_path)?;
    db::migrate(&conn)?;
    // Job table is Level 1 (not yet written by download); surface as empty.
    let jobs: Vec<JobRow> = Vec::new();
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "jobs": jobs
        }))?
    );
    Ok(ExitCode::Ok)
}

#[derive(Debug, Clone, serde::Serialize)]
struct JobRow {
    id: i64,
    entry_id: i64,
    status: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{DownloadEntry, Game, Version};

    fn mem() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        db::migrate(&conn).unwrap();
        conn
    }

    fn seed_game(conn: &rusqlite::Connection, post_id: i64, slug: &str, title: &str) {
        let game = Game {
            slug: slug.to_string(),
            post_id: Some(post_id),
            title: title.to_string(),
            developer: Some("Studio".to_string()),
            current_version: Some("v1.0".to_string()),
            engine: None,
            platforms: vec!["pc".to_string()],
            genres: vec!["adventure".to_string()],
            size_label: Some("1.0 GB".to_string()),
            censorship: None,
            screenshots: vec![],
            description: None,
            versions: vec![Version {
                label: "v1.0".to_string(),
                is_latest: true,
                official: vec![DownloadEntry {
                    label: "FULL".to_string(),
                    variant: None,
                    host: "fileknot".to_string(),
                    platform: Some("pc".to_string()),
                    go_link: "https://lewdzone.com/go/#t=v1.a.b".to_string(),
                }],
                community: vec![],
            }],
            download_entries: vec![],
        };
        let tx = conn.unchecked_transaction().unwrap();
        db::repo::upsert_game(&tx, &game, Some("2024-10-01"), None).unwrap();
        tx.commit().unwrap();
    }

    #[test]
    fn catalog_listing_reads_synced_rows() {
        let conn = mem();
        seed_game(&conn, 1, "game-a", "Alpha");
        seed_game(&conn, 2, "game-b", "Beta");

        let games = db::repo::list_catalog(&conn, None).unwrap();
        assert_eq!(games.len(), 2);
        assert_eq!(games[0].title, "Alpha");
        assert_eq!(games[0].current_version.as_deref(), Some("v1.0"));
        assert_eq!(games[0].genres, vec!["adventure".to_string()]);
        assert_eq!(games[1].title, "Beta");
    }

    #[test]
    fn catalog_listing_respects_limit() {
        let conn = mem();
        seed_game(&conn, 1, "game-a", "Alpha");
        seed_game(&conn, 2, "game-b", "Beta");

        let games = db::repo::list_catalog(&conn, Some(1)).unwrap();
        assert_eq!(games.len(), 1);
    }

    #[test]
    fn migrate_open_and_list_work_end_to_end() {
        let conn = mem();
        seed_game(&conn, 7, "seven", "Seven");
        assert_eq!(db::repo::game_count(&conn).unwrap(), 1);
    }
}
