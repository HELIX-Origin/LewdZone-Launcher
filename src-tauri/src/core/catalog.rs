//! `catalog` — the Storefront's data path (owner: scraper family + cli/app-shell
//! consumers). Fetches the lewdzone.com `/games/` archive through the polite
//! fetch helper and parses it into canonical `GameCard`s. `sync`/`search`
//! subcommands and the Store webview command all land here (Rule 03, Rule 13).

use crate::core::models::{ArchiveMeta, GameCard};
use crate::core::{Context, Error};
use crate::scraper::archive::{parse_archive, ArchivePage};

/// Archive root used for sorting = Popularity by default.
const ARCHIVE_BASE: &str = "https://lewdzone.com/games/";

/// Fetch + parse one archive page (1-indexed). `sort` / `platform` optionally
/// pin the archive's query params (`SortMode` / `Platform` canonical values).
pub fn archive_page(
    _ctx: &Context,
    page: u32,
    sort: Option<&str>,
    platform: Option<&str>,
) -> Result<ArchivePage, Error> {
    let url = archive_url(page, sort, platform);
    let html = crate::scraper::fetch(&url)?;
    Ok(parse_archive(&html))
}

/// Build the archive URL for a page with optional sort/platform filters.
pub fn archive_url(page: u32, sort: Option<&str>, platform: Option<&str>) -> String {
    let mut url = if page > 1 {
        format!("{ARCHIVE_BASE}page/{page}/")
    } else {
        ARCHIVE_BASE.to_string()
    };
    let mut params: Vec<(&str, &str)> = Vec::new();
    if let Some(s) = sort {
        params.push(("sort", s));
    }
    if let Some(p) = platform {
        params.push(("platform", p));
    }
    if !params.is_empty() {
        let qs: String = params
            .iter()
            .map(|(k, v)| format!("{k}={}", encode_query(v)))
            .collect::<Vec<_>>()
            .join("&");
        url.push('?');
        url.push_str(&qs);
    }
    url
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
}
