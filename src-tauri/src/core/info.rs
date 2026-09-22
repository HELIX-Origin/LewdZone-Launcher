//! `info` — show one game's detail (Phase 2, done).
//!
//! Fetches `/game/<slug>/`, parses the canonical `Game`, and prints a single
//! JSON document on stdout. Without `--versions` it prints a compact metadata
//! summary; with `--versions` it prints the full detail including every
//! version and its download entries. The game arg may be a slug, a post id
//! (resolved via the synced DB), or a URL (Rule 12: query commands emit one
//! `--json` document; standard output only, errors to stderr).

use crate::cli::ExitCode;
use crate::core::models::Game;
use crate::core::{Context, Error};
use crate::db;
use crate::scraper;

/// Compact detail view (`info` without `--versions`).
#[derive(Debug, Clone, serde::Serialize)]
pub struct GameSummary {
    pub slug: String,
    pub post_id: Option<i64>,
    pub title: String,
    pub developer: Option<String>,
    pub current_version: Option<String>,
    pub engine: Option<String>,
    pub platforms: Vec<String>,
    pub genres: Vec<String>,
    pub size_label: Option<String>,
    pub censorship: Option<String>,
    pub screenshots: Vec<String>,
    pub description: Option<String>,
    pub version_count: usize,
    pub download_count: usize,
}

impl GameSummary {
    fn summarize(game: &Game) -> Self {
        Self {
            slug: game.slug.clone(),
            post_id: game.post_id,
            title: game.title.clone(),
            developer: game.developer.clone(),
            current_version: game.current_version.clone(),
            engine: game.engine.clone(),
            platforms: game.platforms.clone(),
            genres: game.genres.clone(),
            size_label: game.size_label.clone(),
            censorship: game.censorship.clone(),
            screenshots: game.screenshots.clone(),
            description: game.description.clone(),
            version_count: game.versions.len(),
            download_count: game.download_entries.len(),
        }
    }
}

/// Resolve `game` (slug, post id, or URL) to the canonical slug. A post id is
/// resolved through the synced catalog when the DB exists; a bare slug/URL is
/// normalized directly so `info` works before any sync.
pub fn resolve_slug(ctx: &Context, game: &str) -> Result<String, Error> {
    let trimmed = game.trim();
    if trimmed.is_empty() {
        return Err(Error::Usage("game slug or post id required".to_string()));
    }
    if let Ok(post_id) = trimmed.parse::<i64>() {
        if let Ok(conn) = db::open(&ctx.db_path) {
            if let Ok(Some(slug)) = db::repo::slug_by_post_id(&conn, post_id) {
                return Ok(slug);
            }
            // DB exists but the id is unknown: surface the mismatch, since the
            // user explicitly asked for that id (Rule 12 usage diagnosability).
            return Err(Error::Usage(format!(
                "no game in the catalog with post id {post_id} — run `sync` first"
            )));
        }
    }
    let slug = crate::core::game_arg_slug(trimmed);
    if slug.is_empty() {
        return Err(Error::Usage("game slug or post id required".to_string()));
    }
    Ok(slug)
}

/// Fetch + print one game's detail. `versions` toggles summary vs. full doc.
pub fn run(ctx: &Context, game: &str, versions: bool) -> Result<ExitCode, Error> {
    run_with(ctx, game, versions, &mut |url| scraper::fetch(url))
}

/// Everything except the network seam (tests inject the fixture fetch; Rule 11).
pub fn run_with(
    ctx: &Context,
    game: &str,
    versions: bool,
    fetch: &mut dyn FnMut(&str) -> Result<String, Error>,
) -> Result<ExitCode, Error> {
    let slug = resolve_slug(ctx, game)?;
    let url = format!("https://lewdzone.com/game/{slug}/");
    let html = fetch(&url)?;
    let parsed = scraper::game::parse_game(&html);

    if versions {
        println!("{}", serde_json::to_string_pretty(&parsed)?);
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&GameSummary::summarize(&parsed))?
        );
    }
    Ok(ExitCode::Ok)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Context;

    const GAME_FIXTURE: &str = include_str!("../../tests/fixtures/html/lz_game.html");

    fn ctx() -> Context {
        Context::new(
            std::env::temp_dir().join("lz-info-test.db"),
            std::env::temp_dir().join("lz-info-test.json"),
        )
    }

    fn stub_fetch(url: &str) -> Result<String, Error> {
        assert!(
            url.ends_with("lewdzone.com/game/treasure-of-nadia/"),
            "unexpected url {url}"
        );
        Ok(GAME_FIXTURE.to_string())
    }

    #[test]
    fn info_summary_reflects_fixture() {
        let code = run_with(&ctx(), "treasure-of-nadia", false, &mut stub_fetch).unwrap();
        assert_eq!(code, ExitCode::Ok);
    }

    #[test]
    fn info_derives_summary_fields() {
        let game = scraper::game::parse_game(GAME_FIXTURE);
        let summary = GameSummary::summarize(&game);
        assert_eq!(summary.slug, "treasure-of-nadia");
        assert_eq!(summary.post_id, Some(18212));
        assert_eq!(summary.developer.as_deref(), Some("NLT Media"));
        assert_eq!(
            summary.current_version.as_deref(),
            Some("1.0117 (Finished)")
        );
        assert!(summary.version_count >= 2);
        assert!(summary.download_count >= 1);
    }

    #[test]
    fn info_accepts_slug_url_form() {
        let code = run_with(
            &ctx(),
            "https://lewdzone.com/game/treasure-of-nadia/",
            false,
            &mut stub_fetch,
        )
        .unwrap();
        assert_eq!(code, ExitCode::Ok);
    }

    #[test]
    fn info_rejects_empty_game() {
        let err = run_with(&ctx(), "", false, &mut stub_fetch).unwrap_err();
        assert!(matches!(err, Error::Usage(_)));
    }

    #[test]
    fn info_resolves_post_id_through_db() {
        let conn = db::open(&ctx().db_path).unwrap();
        db::migrate(&conn).unwrap();
        let game = scraper::game::parse_game(GAME_FIXTURE);
        let tx = conn.unchecked_transaction().unwrap();
        db::repo::upsert_game(&tx, &game, Some("2024-10-01"), None).unwrap();
        tx.commit().unwrap();

        let slug = resolve_slug(&ctx(), "18212").unwrap();
        assert_eq!(slug, "treasure-of-nadia");
    }

    #[test]
    fn info_reports_unknown_post_id() {
        let conn = db::open(&ctx().db_path).unwrap();
        db::migrate(&conn).unwrap();
        let err = resolve_slug(&ctx(), "999999").unwrap_err();
        assert!(matches!(err, Error::Usage(_)));
    }
}
