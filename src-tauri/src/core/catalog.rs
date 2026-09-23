//! `catalog` — the Storefront's data path (owner: scraper family + cli/app-shell
//! consumers). Fetches lewdzone.com archive/genre/search pages through the
//! polite fetch helper and parses them into canonical models. `sync`/`search`
//! subcommands and the Store webview commands all land here (Rule 03, Rule 13).

use crate::core::models::{ArchiveFilter, ArchiveMeta, GameCard, Genre};
use crate::core::{Context, Error};
use crate::scraper::archive::{parse_archive, ArchivePage};
use crate::scraper::genres::parse_genres;

/// Archive root used for sorting = Popularity by default.
const ARCHIVE_BASE: &str = "https://lewdzone.com/games/";
/// Genre taxonomy root (`/game-genres/` is the full cloud, each slug has an
/// archive under `/game-genre/<slug>/`).
const GENRE_BASE: &str = "https://lewdzone.com/game-genre/";
/// WordPress site root used for free-text search (`?s=<query>`).
const SEARCH_BASE: &str = "https://lewdzone.com/";

/// Fetch + parse one archive page (1-indexed). `sort` / `platform` optionally
/// pin the archive's query params (`SortMode` / `Platform` canonical values).
pub fn archive_page(
    ctx: &Context,
    page: u32,
    sort: Option<&str>,
    platform: Option<&str>,
) -> Result<ArchivePage, Error> {
    archive_page_filtered(
        ctx,
        page,
        &ArchiveFilter {
            sort: sort.map(str::to_string),
            platform: platform.map(str::to_string),
            ..Default::default()
        },
    )
}

/// Fetch + parse one archive page with the full site filter surface.
pub fn archive_page_filtered(
    _ctx: &Context,
    page: u32,
    filter: &ArchiveFilter,
) -> Result<ArchivePage, Error> {
    let url = archive_url_filtered(page, filter);
    let html = crate::scraper::fetch(&url)?;
    Ok(parse_archive(&html))
}

/// Build the archive URL for a page with optional sort/platform filters.
pub fn archive_url(page: u32, sort: Option<&str>, platform: Option<&str>) -> String {
    archive_url_filtered(
        page,
        &ArchiveFilter {
            sort: sort.map(str::to_string),
            platform: platform.map(str::to_string),
            ..Default::default()
        },
    )
}

/// Build the archive URL with the full filter surface. Mirrors the site's
/// GET form 1:1 (Rule 13: the Store view and the CLI share this shape):
/// `q`, `platform`, `engine`, `state`, `sort`, repeated `tags[]` and
/// `tags-exclude[]`.
pub fn archive_url_filtered(page: u32, filter: &ArchiveFilter) -> String {
    let mut url = if page > 1 {
        format!("{ARCHIVE_BASE}page/{page}/")
    } else {
        ARCHIVE_BASE.to_string()
    };
    let mut params: Vec<String> = Vec::new();
    if let Some(q) = filter.q.as_deref() {
        params.push(format!("q={}", encode_query(q)));
    }
    if let Some(p) = filter.platform.as_deref() {
        params.push(format!("platform={}", encode_query(p)));
    }
    if let Some(e) = filter.engine.as_deref() {
        params.push(format!("engine={}", encode_query(e)));
    }
    if let Some(st) = filter.state.as_deref() {
        params.push(format!("state={}", encode_query(st)));
    }
    if let Some(s) = filter.sort.as_deref() {
        params.push(format!("sort={}", encode_query(s)));
    }
    for tag in &filter.include_tags {
        params.push(format!("tags%5B%5D={}", encode_query(tag)));
    }
    for tag in &filter.exclude_tags {
        params.push(format!("tags-exclude%5B%5D={}", encode_query(tag)));
    }
    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }
    url
}

/// Fetch + parse the full genre cloud (`/game-genres/`).
pub fn genres_page(_ctx: &Context) -> Result<Vec<Genre>, Error> {
    let html = crate::scraper::fetch(GENRE_BASE.trim_end_matches('/'))?;
    Ok(parse_genres(&html))
}

/// Fetch + parse one genre archive page (1-indexed). `sort` optionally pins
/// the filter bar's Sort By select.
pub fn genre_page(
    _ctx: &Context,
    slug: &str,
    page: u32,
    sort: Option<&str>,
) -> Result<ArchivePage, Error> {
    let url = genre_url(slug, page, sort);
    let html = crate::scraper::fetch(&url)?;
    Ok(parse_archive(&html))
}

/// Build a genre archive URL: `/game-genre/<slug>/` (+ `/page/N/` + `?sort=`).
pub fn genre_url(slug: &str, page: u32, sort: Option<&str>) -> String {
    let mut url = if page > 1 {
        format!("{GENRE_BASE}{slug}/page/{page}/")
    } else {
        format!("{GENRE_BASE}{slug}/")
    };
    if let Some(s) = sort {
        url.push_str(&format!("?sort={}", encode_query(s)));
    }
    url
}

/// Build the WordPress free-text search URL: `https://lewdzone.com/?s=<q>`.
pub fn search_url(query: &str) -> String {
    format!("{}?s={}", SEARCH_BASE, encode_query(query))
}

/// Query-string encode with the + convention the site uses for spaces.
fn encode_query(v: &str) -> String {
    v.replace(' ', "+")
}

/// Convenience when the caller only wants the cards.
pub fn archive_cards(
    ctx: &Context,
    page: u32,
    sort: Option<&str>,
    platform: Option<&str>,
) -> Result<Vec<GameCard>, Error> {
    Ok(archive_page(ctx, page, sort, platform)?.games)
}

/// Empty-metadata helper so callers that don't need page stats can still get a
/// typed `ArchiveMeta` for boundary serialization.
pub fn empty_meta(page: u32) -> ArchiveMeta {
    ArchiveMeta {
        page,
        total_pages: None,
        applied: std::collections::BTreeMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_one_url_is_the_archive_root() {
        assert_eq!(archive_url(1, None, None), "https://lewdzone.com/games/");
    }

    #[test]
    fn later_pages_use_page_segment() {
        assert_eq!(
            archive_url(2, None, None),
            "https://lewdzone.com/games/page/2/"
        );
    }

    #[test]
    fn sort_param_uses_plus_for_spaces() {
        assert_eq!(
            archive_url(1, Some("New to Old"), None),
            "https://lewdzone.com/games/?sort=New+to+Old"
        );
    }

    #[test]
    fn platform_and_sort_combined() {
        let url = archive_url(3, Some("Popularity"), Some("PC"));
        assert!(
            url.starts_with("https://lewdzone.com/games/page/3/?")
                && url.contains("sort=Popularity")
                && url.contains("platform=PC")
        );
    }

    #[test]
    fn full_filter_serializes_every_site_param() {
        let url = archive_url_filtered(
            1,
            &ArchiveFilter {
                q: Some("treasure of nadia".to_string()),
                platform: Some("PC".to_string()),
                engine: Some("RenPy".to_string()),
                state: Some("Finished".to_string()),
                sort: Some("Popularity".to_string()),
                include_tags: vec!["rpg".to_string(), "harem".to_string()],
                exclude_tags: vec!["incest".to_string()],
            },
        );
        assert!(url.starts_with("https://lewdzone.com/games/?"));
        assert!(url.contains("q=treasure+of+nadia"));
        assert!(url.contains("platform=PC"));
        assert!(url.contains("engine=RenPy"));
        assert!(url.contains("state=Finished"));
        assert!(url.contains("sort=Popularity"));
        assert!(url.contains("tags%5B%5D=rpg"));
        assert!(url.contains("tags%5B%5D=harem"));
        assert!(url.contains("tags-exclude%5B%5D=incest"));
    }

    #[test]
    fn tagged_page_uses_page_segment() {
        let url = archive_url_filtered(
            2,
            &ArchiveFilter {
                include_tags: vec!["adventure".to_string()],
                ..Default::default()
            },
        );
        assert!(url.starts_with("https://lewdzone.com/games/page/2/?tags%5B%5D=adventure"));
    }

    #[test]
    fn archive_page_parses_the_fixture_offline() {
        // Rule 11 / Rule 05: no network. The parse comes from the committed
        // fixture via the archive parser's own tests; here we assert only that
        // the module composes URL + parse without a live fetch.
        let html = include_str!("../../tests/fixtures/html/lz_archive.html");
        let parsed = parse_archive(html);
        assert_eq!(parsed.games.len(), 20);
        assert_eq!(parsed.meta.page, 1);
        assert_eq!(parsed.meta.total_pages, Some(787));
    }

    #[test]
    fn genre_url_shapes_are_canonical() {
        assert_eq!(
            genre_url("3d-games", 1, None),
            "https://lewdzone.com/game-genre/3d-games/"
        );
        assert_eq!(
            genre_url("3d-games", 3, None),
            "https://lewdzone.com/game-genre/3d-games/page/3/"
        );
        assert_eq!(
            genre_url("3d-games", 1, Some("New to Old")),
            "https://lewdzone.com/game-genre/3d-games/?sort=New+to+Old"
        );
    }

    #[test]
    fn search_url_uses_wordpress_query() {
        assert_eq!(
            search_url("treasure of nadia"),
            "https://lewdzone.com/?s=treasure+of+nadia"
        );
        assert_eq!(search_url("titanic"), "https://lewdzone.com/?s=titanic");
    }

    #[test]
    fn genre_cloud_parses_fixture_offline() {
        let html = include_str!("../../tests/fixtures/html/lz_genres.html");
        let genres = parse_genres(html);
        assert!(genres.len() >= 100);
        assert!(genres.iter().any(|g| g.slug == "2d-game"));
    }

    #[test]
    fn search_results_parse_fixture_offline() {
        let html = include_str!("../../tests/fixtures/html/lz_search.html");
        let parsed = parse_archive(html);
        assert_eq!(parsed.games.len(), 1);
    }
}
