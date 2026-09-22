//! Archive parser — turns a `/games/` listing page into canonical models
//! (owner: `scraper/archive-scraper`). 100% pure: `(html) -> models`, no
//! network, no DB. Built against fixture `tests/fixtures/html/lz_archive.html`.
//!
//! Site map (verified 2026-09-20):
//! - Cards: `div.item.games.detailed`, one `.col-md-12` block apiece.
//! - Left col: `a[href="/game/<slug>/"] > .img-wrap > .previewList > img.preview[data-src]`
//!   (first preview = cover thumb), `.imageCount`, `.gameState[title]`.
//! - Right col: `.detailedBody`, `h4[title]`, `.version-tag`, `<small> by X`,
//!   `.content`, `.taglist a[rel=tag]` (genre name + class = slug), `.post-meta`
//!   `.engine`, `.entry-platform i.fab`, two `.entry-date`, `.entry-view`.
//! - Pagination: `.wp-pagenavi .pages` = "Page 1 of 787"; `a.page.larger`
//!   links use `/games/page/N/?sort=Popularity`.

use crate::core::models::{normalize_platform, ArchiveMeta, GameCard};
use scraper::{Html, Selector};

/// Everything parsed off one archive page.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ArchivePage {
    pub games: Vec<GameCard>,
    pub meta: ArchiveMeta,
}

/// Parse a `/games/` HTML document into cards + page metadata.
pub fn parse_archive(html: &str) -> ArchivePage {
    let document = Html::parse_document(html);
    ArchivePage {
        games: parse_cards(&document),
        meta: parse_meta(&document),
    }
}

/// Parse just the game cards (empty = no listing / unexpected layout).
fn parse_cards(document: &Html) -> Vec<GameCard> {
    let card_sel = Selector::parse("div.item.games.detailed").expect("valid card selector");
    let link_sel =
        Selector::parse("a[href^='https://lewdzone.com/game/']").expect("valid game-link selector");
    let img_sel = Selector::parse("img.preview[data-src]").expect("valid img selector");
    let h4_sel = Selector::parse("h4[title]").expect("valid h4 selector");
    let vt_sel = Selector::parse(".version-tag").expect("valid vt selector");
    let by_sel = Selector::parse("small").expect("valid small selector");
    let content_sel = Selector::parse(".content").expect("valid content selector");
    let genre_sel = Selector::parse("a[rel='tag']").expect("valid genre selector");
    let engine_sel = Selector::parse(".engine").expect("valid engine selector");
    let plat_sel = Selector::parse(".entry-platform i").expect("valid plat selector");
    let state_sel = Selector::parse(".gameState[title]").expect("valid state selector");
    let updated_sel = Selector::parse(".entry-date[title='date of update in Lewdzone']")
        .expect("valid updated selector");
    let views_sel = Selector::parse(".entry-view").expect("valid views selector");

    let mut cards = Vec::new();
    for node in document.select(&card_sel) {
        let href = node
            .select(&link_sel)
            .next()
            .and_then(|a| a.value().attr("href"))
            .map(str::to_string);
        let slug = href.as_deref().and_then(slug_from_game_url);

        let thumb = node
            .select(&img_sel)
            .next()
            .and_then(|img| img.value().attr("data-src"))
            .map(str::to_string);

        let title = node
            .select(&h4_sel)
            .next()
            .and_then(|h4| h4.value().attr("title"))
            .map(clean_text);

        let version_tag = node
            .select(&vt_sel)
            .next()
            .map(|e| clean_text(&e.text().collect::<String>()));

        let developer = node
            .select(&by_sel)
            .next()
            .map(|e| clean_text(&e.text().collect::<String>()))
            .and_then(|txt| txt.strip_prefix("by ").map(|s| s.trim().to_string()))
            .filter(|s| !s.is_empty());

        let description = node
            .select(&content_sel)
            .next()
            .map(|e| clean_text(&e.text().collect::<String>()));

        let mut genres = Vec::new();
        let mut genre_slugs = Vec::new();
        for a in node.select(&genre_sel) {
            if let Some(name) = a.text().next() {
                let name = clean_text(name);
                if !name.is_empty() {
                    genres.push(name);
                }
            }
            if let Some(href) = a.value().attr("href") {
                if let Some(slug) = slug_from_genre_url(href) {
                    genre_slugs.push(slug);
                }
            }
        }

        let engine = node
            .select(&engine_sel)
            .next()
            .map(|e| clean_text(&e.text().collect::<String>()));

        let platforms = node
            .select(&plat_sel)
            .filter_map(|i| i.value().attr("class"))
            .flat_map(|c| c.split_whitespace())
            .map(normalize_platform)
            .filter(|p| p != "fab")
            .collect::<Vec<_>>();

        let state = node
            .select(&state_sel)
            .next()
            .and_then(|s| s.value().attr("title"))
            .map(str::to_string)
            .filter(|s| !s.is_empty());

        let updated_at = node
            .select(&updated_sel)
            .next()
            .map(|e| clean_text(&e.text().collect::<String>()));

        let views = node
            .select(&views_sel)
            .next()
            .map(|e| clean_text(&e.text().collect::<String>()));

        // A card without a game href or a title isn't a games card (ad slot,
        // widget, etc.) — skip it rather than emit a hollow struct.
        if slug.is_none() && title.is_none() {
            continue;
        }

        cards.push(GameCard {
            slug: slug.unwrap_or_default(),
            title: title.unwrap_or_default(),
            post_id: None,
            thumb_url: thumb,
            platforms,
            engine,
            state,
            version_tag,
            developer,
            description,
            genres,
            genre_slugs,
            updated_at,
            views,
        });
    }
    cards
}

/// Parse the pagination + filter metadata off an archive page.
fn parse_meta(document: &Html) -> ArchiveMeta {
    let pages_sel = Selector::parse(".wp-pagenavi .pages").expect("valid pages selector");
    let current_sel = Selector::parse(".wp-pagenavi .current").expect("valid current selector");
    let applied_sel = Selector::parse("select option[selected]").expect("valid filter selector");

    let mut meta = ArchiveMeta {
        page: 1,
        total_pages: None,
        applied: Default::default(),
    };

    if let Some(pages) = document.select(&pages_sel).next() {
        let text = pages.text().collect::<String>();
        // "Page 1 of 787"
        let mut parts = text
            .split_whitespace()
            .filter_map(|w| w.parse::<u32>().ok());
        if let Some(page) = parts.next() {
            meta.page = page;
        }
        if let Some(total) = parts.next() {
            meta.total_pages = Some(total);
        }
    } else if let Some(current) = document.select(&current_sel).next() {
        // Fallback: `.current` holds the numeric page.
        meta.page = current
            .text()
            .collect::<String>()
            .trim()
            .parse::<u32>()
            .unwrap_or(1);
    }

    // Active filter selects, e.g. `<option value="Popularity" selected>`.
    for option in document.select(&applied_sel) {
        let name = option
            .parent()
            .and_then(|p| p.value().as_element())
            .and_then(|el| el.attr("name"))
            .unwrap_or("filter");
        let value = option.value().attr("value").unwrap_or_default().to_string();
        meta.applied.insert(name.to_string(), value);
    }

    meta
}

/// `https://lewdzone.com/game/wild-life/` -> `wild-life`.
fn slug_from_game_url(url: &str) -> Option<String> {
    let trimmed = url.trim_end_matches('/');
    let slug = trimmed.rsplit('/').next()?;
    if slug.is_empty() || slug == "game" {
        None
    } else {
        Some(slug.to_string())
    }
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

    const FIXTURE: &str = include_str!("../../tests/fixtures/html/lz_archive.html");

    fn parsed() -> ArchivePage {
        parse_archive(FIXTURE)
    }

    #[test]
    fn parses_all_cards_on_page_one() {
        let page = parsed();
        assert!(!page.games.is_empty(), "expected game cards");
        assert_eq!(page.games.len(), 20, "page 1 fixture holds 20 cards");
    }

    #[test]
    fn first_card_is_wild_life_with_full_fields() {
        let page = parsed();
        let first = &page.games[0];
        assert_eq!(first.slug, "wild-life");
        assert_eq!(first.title, "Wild Life");
        assert_eq!(first.version_tag.as_deref(), Some("v2026-06-15 Full"));
        assert_eq!(first.developer.as_deref(), Some("Adeptus Steve"));
        assert_eq!(first.engine.as_deref(), Some("Unreal Engine"));
        assert_eq!(first.state.as_deref(), Some("Ongoing"));
        assert_eq!(first.platforms, vec!["pc"]);
        assert_eq!(first.updated_at.as_deref(), Some("June 18, 2026"));
        assert_eq!(first.views.as_deref(), Some("962K"));
        assert!(first
            .thumb_url
            .as_deref()
            .unwrap_or_default()
            .contains("Wild-Life-Adult-Game-XXX-Cover-329x196.jpg"));
        assert!(first.genres.iter().any(|g| g == "3D Game" || g == "3DCG"));
        assert!(first.genre_slugs.iter().any(|s| s == "3d-games"));
        assert!(first
            .description
            .as_deref()
            .unwrap_or_default()
            .contains("mad universe"));
    }

    #[test]
    fn multi_platform_cards_normalize() {
        let page = parsed();
        let ram = page
            .games
            .iter()
            .find(|g| g.slug == "rick-and-morty-a-way-back-home")
            .expect("rick-and-morty present");
        assert_eq!(ram.platforms, vec!["pc", "android", "mac", "linux"]);
        assert_eq!(ram.developer.as_deref(), Some("Ferdafs"));
        assert_eq!(ram.state.as_deref(), Some("Ongoing"));
    }

    #[test]
    fn finished_card_reports_state() {
        let page = parsed();
        let dmd = page
            .games
            .iter()
            .find(|g| g.slug == "dating-my-daughter")
            .expect("dating-my-daughter present");
        assert_eq!(dmd.state.as_deref(), Some("Finished"));
    }

    #[test]
    fn pagination_meta_reads_page_and_total() {
        let meta = parsed().meta;
        assert_eq!(meta.page, 1);
        assert_eq!(meta.total_pages, Some(787));
    }

    #[test]
    fn filter_selects_are_detected() {
        let meta = parsed().meta;
        assert!(!meta.applied.is_empty());
        assert_eq!(
            meta.applied.get("sort").map(|s| s.as_str()),
            Some("Popularity")
        );
    }

    #[test]
    fn slug_helpers_extract_clean_ids() {
        assert_eq!(
            slug_from_game_url("https://lewdzone.com/game/wild-life/"),
            Some("wild-life".into())
        );
        assert_eq!(
            slug_from_genre_url("https://lewdzone.com/game-genre/3d-games/"),
            Some("3d-games".into())
        );
        assert!(slug_from_game_url("https://lewdzone.com/game/").is_none());
    }
}
