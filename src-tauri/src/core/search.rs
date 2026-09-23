//! `search` — browse or search the lewdzone.com archive.
//! Shares `core::catalog` with the Store view (Rule 03 GUI/CLI parity).
//!
//! Free-text search maps to WordPress `/?s=<query>` (verified 2026-09-22:
//! one result page for "titanic", `div.item.games.detailed` cards, cover in
//! `img.preview[src]` — the archive parser's `src` fallback covers it). A
//! bare `search` shows the Popularity front page instead (browse fallback).

use crate::cli::ExitCode;
use crate::core::{catalog, Context, Error};

pub fn run(ctx: &Context, query: &str) -> Result<ExitCode, Error> {
    let parsed = if query.trim().is_empty() {
        catalog::archive_page(ctx, 1, Some("Popularity"), None)?
    } else {
        let url = catalog::search_url(query.trim());
        let html = crate::scraper::fetch(&url)?;
        crate::scraper::archive::parse_archive(&html)
    };
    println!("{}", serde_json::to_string_pretty(&parsed)?);
    Ok(ExitCode::Ok)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_text_search_parses_wordpress_results_offline() {
        // Rule 11 / Rule 05: no network. The `?s=` HTML comes from the
        // committed search fixture and the archive parser pinged the pipe.
        let html = include_str!("../../tests/fixtures/html/lz_search.html");
        let parsed = crate::scraper::archive::parse_archive(html);
        assert_eq!(parsed.games.len(), 1);
        assert!(parsed.games[0].thumb_url.is_some());
    }

    #[test]
    fn search_url_builds_the_query_or_archive() {
        assert_eq!(
            catalog::search_url("titanic"),
            "https://lewdzone.com/?s=titanic"
        );
        assert_eq!(
            catalog::archive_url(1, Some("Popularity"), None),
            "https://lewdzone.com/games/?sort=Popularity"
        );
        assert_eq!(
            catalog::archive_url(2, None, Some("PC")),
            "https://lewdzone.com/games/page/2/?platform=PC"
        );
    }
}
