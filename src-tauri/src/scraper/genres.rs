//! Genre cloud parser — turns the `/game-genres/` tag-cloud page into typed
//! genre entries (owner: `scraper/archive-scraper`). 100% pure: `(html) ->
//! models`, no network, no DB. Built against fixture
//! `tests/fixtures/html/lz_genres.html`.
//!
//! Site map (verified 2026-09-22):
//! - `div.container.site-margin.tag-cloud`
//! - Rows: `div.col-6.col-sm-4.col-md-3.col-lg-2`
//! - Inside each: `a[href="https://lewdzone.com/game-genre/<slug>/"]`
//!   label text + `span` count (plain digits), e.g.
//!   `2D Game <span>5641</span>`.

use crate::core::models::Genre;
use scraper::{Html, Selector};

/// Parse the `<div class="container site-margin tag-cloud">` genre cloud.
/// Returns entries in page order (the site sorts these by slug within a
/// column).
pub fn parse_genres(html: &str) -> Vec<Genre> {
    let document = Html::parse_document(html);
    let cell_sel =
        Selector::parse("div.col-6.col-sm-4.col-md-3.col-lg-2").expect("valid cell selector");
    let link_sel = Selector::parse("a[href^='https://lewdzone.com/game-genre/']")
        .expect("valid link selector");
    let span_sel = Selector::parse("span").expect("valid span selector");

    let mut genres = Vec::new();
    for cell in document.select(&cell_sel) {
        let Some(link) = cell.select(&link_sel).next() else {
            continue;
        };
        let Some(href) = link.value().attr("href") else {
            continue;
        };
        let Some(slug) = slug_from_genre_url(href) else {
            continue;
        };
        // Label = the anchor's text minus the trailing count span.
        let mut label = clean_text(&link.text().collect::<String>());
        if let Some(span) = link.select(&span_sel).next() {
            let count_text = span.text().collect::<String>();
            label = clean_text(&label.replace(&count_text, ""));
        }
        let count = link
            .select(&span_sel)
            .next()
            .and_then(|s| s.text().next())
            .and_then(|t| t.trim().parse::<u32>().ok());
        genres.push(Genre { label, slug, count });
    }
    genres
}

/// `https://lewdzone.com/game-genre/3d-games/` -> `3d-games`.
fn slug_from_genre_url(url: &str) -> Option<String> {
    let trimmed = url.trim_end_matches('/');
    let slug = trimmed.rsplit('/').next()?;
    if slug.is_empty() || slug == "game-genre" {
        None
    } else {
        Some(slug.to_string())
    }
}

/// Collapse whitespace/newlines in scraped text nodes.
fn clean_text(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../../tests/fixtures/html/lz_genres.html");

    #[test]
    fn parses_the_genre_cloud_offline() {
        let genres = parse_genres(FIXTURE);
        assert!(!genres.is_empty(), "expected genre entries");
        assert!(genres.len() >= 100, "cloud fixture holds many genres");
    }

    #[test]
    fn first_genres_have_label_slug_and_count() {
        let genres = parse_genres(FIXTURE);
        let first = genres
            .iter()
            .find(|g| g.slug == "2d-game")
            .expect("2d-game present");
        assert_eq!(first.label, "2D Game");
        assert_eq!(first.count, Some(5641));
        let second = genres
            .iter()
            .find(|g| g.slug == "2dcg")
            .expect("2dcg present");
        assert_eq!(second.label, "2DCG");
        assert_eq!(second.count, Some(12136));
    }

    #[test]
    fn ad_slots_are_skipped() {
        let genres = parse_genres(FIXTURE);
        assert!(
            genres
                .iter()
                .all(|g| !g.label.is_empty() && !g.slug.is_empty()),
            "no hollow genre entries"
        );
    }
}
