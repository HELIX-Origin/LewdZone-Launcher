//! Game-page parser — turns a `/game/<slug>/` document into a canonical
//! `Game` profile (owner: `scraper/game-page-scraper`). 100% pure:
//! `(html) -> models`, no network, no DB. Built against fixture
//! `tests/fixtures/html/lz_game.html` (treasure-of-nadia).
//!
//! Site map (verified 2026-09-20):
//! - Header: `h1.post-title`, `div.wp-postratings[data-post-id]`,
//!   left column `<p>` pairs "Game Size: 7.51 GB", "Censorship: Uncensored",
//!   "Game Engine: RPGM", "Current Game Version: 1.0117 (Finished)",
//!   dev anchor under "Developer: ".
//! - Genres: first `div.taglist a[rel=tag]` (name text + `/game-genre/<slug>/`).
//! - Screenshots: `div.gallery figure.gallery-item .gallery-icon a[href]`
//!   (full-res URL is the anchor href).
//! - Versions: `#lz-version-select option[value]` — first non-empty value is
//!   the page's current version, older ones are `?ver=<X>#sys_req` pages.
//! - Downloads: `#server-tab-pane` (official) and `#alt-tab-pane`
//!   (community); each holds `<span class="windows|android|mac|linux">`
//!   platform markers followed by `a.downloadLink.d-<host>[...]` rows.

use crate::core::models::{normalize_platform, DownloadEntry, Game, Version};
use scraper::{ElementRef, Html, Selector};

/// Parse a `/game/<slug>/` HTML document into a `Game` profile.
pub fn parse_game(html: &str) -> Game {
    let document = Html::parse_document(html);

    let headline = first_text(&document, "h1.post-title").unwrap_or_default();
    let slug = canonical_slug(&document)
        .or_else(|| meta(&document, "og:url").and_then(|u| slug_from_url(&u)))
        .unwrap_or_default();
    let post_id = post_id(&document);

    let header_fields = parse_header_fields(&document);

    let developer = dev_name(&document).or_else(|| header_fields.get("Developer").cloned());
    let current_version = header_fields
        .get("Current Game Version")
        .or_else(|| header_fields.get("Game Version"))
        .cloned();
    let engine = header_fields.get("Game Engine").cloned();
    let size_label = header_fields.get("Game Size").cloned();
    let censorship = header_fields.get("Censorship").cloned();

    let genres = parse_genres(&document);
    let screenshots = parse_screenshots(&document);
    let description = meta(&document, "description");

    let versions = parse_versions(&document);
    let (official, community) = parse_download_tabs(&document);

    let mut platforms = Vec::new();
    for entry in official.iter().chain(community.iter()) {
        if let Some(p) = &entry.platform {
            if !platforms.contains(p) {
                platforms.push(p.clone());
            }
        }
    }

    let mut download_entries = Vec::new();
    let mut versions_out = Vec::new();
    if versions.is_empty() {
        if !official.is_empty() || !community.is_empty() {
            let label = current_version
                .as_deref()
                .map(|v| v.trim())
                .filter(|v| !v.is_empty())
                .unwrap_or("latest")
                .to_string();
            let v = Version {
                label,
                is_latest: true,
                official: official.clone(),
                community: community.clone(),
            };
            download_entries.extend(official);
            download_entries.extend(community);
            versions_out.push(v);
        }
    } else {
        if let Some(mut first) = versions.first().cloned() {
            first.official = official;
            first.community = community;
            download_entries.extend(first.official.iter().cloned());
            download_entries.extend(first.community.iter().cloned());
            versions_out.push(first);
        }
        for version in versions.iter().skip(1).cloned() {
            download_entries.extend(version.official.iter().cloned());
            download_entries.extend(version.community.iter().cloned());
            versions_out.push(version);
        }
    }

    Game {
        slug,
        post_id,
        title: clean_text(&headline),
        developer,
        current_version,
        engine,
        platforms,
        genres,
        size_label,
        censorship,
        screenshots,
        description,
        versions: versions_out,
        download_entries,
    }
}

/// Extract a `Key: Value` map from the header `<p>` blocks (Game Size,
/// Censorship, Game Engine, Current Game Version, Release Date, ...).
fn parse_header_fields(document: &Html) -> std::collections::BTreeMap<String, String> {
    let mut map = std::collections::BTreeMap::new();
    let p_sel = Selector::parse("div.content-block p, div.content-block > div p")
        .expect("valid header p selector");
    for p in document.select(&p_sel) {
        let text = clean_text(&p.text().collect::<String>());
        if let Some((key, value)) = text.split_once(':') {
            let key = key.trim().to_string();
            let value = value.trim().to_string();
            if !key.is_empty() && !value.is_empty() {
                map.insert(key, value);
            }
        }
    }
    map
}

/// Developer name from the "Developer: <a>Name</a>" anchor (Patreon link
/// typically) or a `<strong>Developer: ...` text.
fn dev_name(document: &Html) -> Option<String> {
    let a_sel = Selector::parse("p.dev a, .info-group a[href*='patreon']").ok()?;
    let p_sel = Selector::parse("p.dev").ok()?;
    if let Some(a) = document.select(&a_sel).next() {
        let name = clean_text(&a.text().collect::<String>());
        if !name.is_empty() {
            return Some(name);
        }
    }
    document
        .select(&p_sel)
        .next()
        .map(|el| clean_text(&el.text().collect::<String>()))
        .and_then(|t| {
            t.split_once("Developer:")
                .map(|(_, rest)| rest.trim().to_string())
        })
        .filter(|s| !s.is_empty())
}

fn post_id(document: &Html) -> Option<i64> {
    let sel = Selector::parse(".wp-postratings [data-post-id], .wp-postratings[data-post-id]")
        .expect("valid rating selector");
    if let Some(id) = document
        .select(&sel)
        .next()
        .and_then(|el| el.value().attr("data-post-id"))
        .and_then(|v| v.parse().ok())
    {
        return Some(id);
    }

    // Fallback 1: <link rel="shortlink" href="https://lewdzone.com/?p=18212">
    if let Ok(shortlink_sel) = Selector::parse("link[rel='shortlink']") {
        if let Some(href) = document
            .select(&shortlink_sel)
            .next()
            .and_then(|el| el.value().attr("href"))
        {
            if let Some((_, p)) = href.split_once("?p=") {
                let digits: String = p.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(id) = digits.parse::<i64>() {
                    return Some(id);
                }
            }
        }
    }

    // Fallback 2: <body class="... postid-18212 ...">
    if let Ok(body_sel) = Selector::parse("body") {
        if let Some(class_attr) = document
            .select(&body_sel)
            .next()
            .and_then(|el| el.value().attr("class"))
        {
            for token in class_attr.split_whitespace() {
                if let Some(num_str) = token.strip_prefix("postid-") {
                    if let Ok(id) = num_str.parse::<i64>() {
                        return Some(id);
                    }
                }
            }
        }
    }

    None
}

fn parse_genres(document: &Html) -> Vec<String> {
    let sel = Selector::parse("div.taglist a[rel='tag']").expect("valid genre selector");
    document
        .select(&sel)
        .filter_map(|a| {
            let name = clean_text(&a.text().collect::<String>());
            if name.is_empty() {
                None
            } else {
                Some(name)
            }
        })
        .collect()
}

/// Parse the authoritative cover image URL for a game page.
/// Checks (in priority order):
/// 1. CSS `--cover-background: url('...')` (the exact thumbnail used on store cards)
/// 2. `<meta property="og:image:url">` / `<meta property="og:image">`
/// 3. First screenshot from gallery
pub fn parse_cover_url(html: &str) -> Option<String> {
    let document = Html::parse_document(html);

    // 1. Check style tag for --cover-background
    if let Ok(style_sel) = Selector::parse("style") {
        for style in document.select(&style_sel) {
            let text = style.text().collect::<String>();
            if let Some(pos) = text.find("--cover-background") {
                let rest = &text[pos..];
                if let Some(url_start) = rest.find("url(") {
                    let after_url = &rest[url_start + 4..];
                    if let Some(url_end) = after_url.find(')') {
                        let raw_url = after_url[..url_end]
                            .trim()
                            .trim_matches('\'')
                            .trim_matches('"');
                        if !raw_url.is_empty() && raw_url.starts_with("http") {
                            return Some(raw_url.to_string());
                        }
                    }
                }
            }
        }
    }

    // 2. OpenGraph / Twitter meta image
    let meta_keys = ["og:image:url", "og:image", "twitter:image"];
    for key in meta_keys {
        if let Some(u) = meta(&document, key) {
            if !u.is_empty() && u.starts_with("http") {
                return Some(u);
            }
        }
    }

    // 3. First screenshot
    let screenshots = parse_screenshots(&document);
    screenshots.into_iter().next()
}

fn parse_screenshots(document: &Html) -> Vec<String> {
    let mut urls = Vec::new();

    // 1. Primary gallery thumbnail anchor links (points to full-res original)
    let gallery_selectors = [
        "div.gallery figure.gallery-item .gallery-icon a[href]",
        "div.gallery .gallery-icon a[href]",
        "div.gallery figure a[href]",
        "div.gallery a[href]",
        "figure.wp-block-gallery a[href]",
        "ul.wp-block-gallery a[href]",
        ".blocks-gallery-item a[href]",
    ];
    for sel_str in gallery_selectors {
        if let Ok(sel) = Selector::parse(sel_str) {
            for a in document.select(&sel) {
                if let Some(href) = a.value().attr("href") {
                    let href_lower = href.to_ascii_lowercase();
                    if href_lower.contains("wp-content/uploads")
                        && (href_lower.contains(".jpg")
                            || href_lower.contains(".jpeg")
                            || href_lower.contains(".png")
                            || href_lower.contains(".webp")
                            || href_lower.contains(".gif"))
                        && !urls.contains(&href.to_string())
                    {
                        urls.push(href.to_string());
                    }
                }
            }
        }
    }

    // 2. Carousel, slider, and content preview images (when gallery anchor links are absent)
    if urls.is_empty() {
        let img_selectors = [
            ".carousel img",
            ".swiper-slide img",
            "figure.wp-block-gallery img",
            ".blocks-gallery-item img",
            "div.gallery figure.gallery-item img",
            "div.gallery img",
            "div.main-content img",
            "div.entry-content img",
            "div.content-block img",
        ];
        for sel_str in img_selectors {
            if let Ok(sel) = Selector::parse(sel_str) {
                for img in document.select(&sel) {
                    let src = img
                        .value()
                        .attr("data-full-url")
                        .or_else(|| img.value().attr("data-src"))
                        .or_else(|| img.value().attr("src"));
                    if let Some(s) = src {
                        let s_lower = s.to_ascii_lowercase();
                        if s_lower.contains("wp-content/uploads")
                            && !s_lower.contains("favicon")
                            && !s_lower.contains("logo")
                            && !s_lower.contains("avatar")
                            && !s_lower.contains("adzone")
                            && (s_lower.contains(".jpg")
                                || s_lower.contains(".jpeg")
                                || s_lower.contains(".png")
                                || s_lower.contains(".webp")
                                || s_lower.contains(".gif"))
                            && !urls.contains(&s.to_string())
                        {
                            urls.push(s.to_string());
                            if urls.len() >= 10 {
                                break;
                            }
                        }
                    }
                }
            }
            if urls.len() >= 10 {
                break;
            }
        }
    }

    // Deduplicate preserving order and cap at 10
    let mut deduped = Vec::new();
    for u in urls {
        if !deduped.contains(&u) {
            deduped.push(u);
            if deduped.len() >= 10 {
                break;
            }
        }
    }
    deduped
}

/// Versions from the `#lz-version-select` dropdown. First non-empty option is
/// the page's current version (`is_latest = true`); the rest are alternative
/// `?ver=` pages whose entries need a separate fetch.
fn parse_versions(document: &Html) -> Vec<Version> {
    let opt_sel =
        Selector::parse("#lz-version-select option[value]").expect("valid option selector");
    let mut versions = Vec::new();
    for opt in document.select(&opt_sel) {
        let value = opt.value().attr("value").unwrap_or_default();
        if value.is_empty() {
            continue;
        }
        let label = clean_text(&opt.text().collect::<String>());
        if label.is_empty() {
            continue;
        }
        versions.push(Version {
            label,
            is_latest: versions.is_empty(),
            official: Vec::new(),
            community: Vec::new(),
        });
    }
    versions
}

/// Download rows split by tab: (official, community).
fn parse_download_tabs(document: &Html) -> (Vec<DownloadEntry>, Vec<DownloadEntry>) {
    let official_sel = Selector::parse("#server-tab-pane").expect("valid official pane selector");
    let alt_sel = Selector::parse("#alt-tab-pane").expect("valid alt pane selector");
    let official = document
        .select(&official_sel)
        .next()
        .map(parse_pane_entries)
        .unwrap_or_default();
    let community = document
        .select(&alt_sel)
        .next()
        .map(parse_pane_entries)
        .unwrap_or_default();
    (official, community)
}

/// Parse `<span class="windows|android|mac">` platform markers + following
/// `a.downloadLink` rows in document order.
fn parse_pane_entries(pane: ElementRef) -> Vec<DownloadEntry> {
    let mixed_sel =
        Selector::parse("a.downloadLink, span.windows, span.android, span.mac, span.linux")
            .expect("valid mixed pane selector");

    let mut entries = Vec::new();
    let mut current_platform: Option<String> = None;

    for node in pane.select(&mixed_sel) {
        let name = node.value().name();
        if name == "span" {
            if let Some(classes) = node.value().attr("class") {
                for token in classes.split_whitespace() {
                    let normalized = normalize_platform(token);
                    if matches!(normalized.as_str(), "pc" | "android" | "mac" | "linux") {
                        current_platform = Some(normalized);
                        break;
                    }
                }
            }
            continue;
        }
        if name != "a" {
            continue;
        }

        let go_link = node.value().attr("href").unwrap_or_default().to_string();
        if go_link.is_empty() || !go_link.contains("lewdzone.com/go/") {
            continue;
        }
        let host = host_from_classes(node);
        let label = clean_text(&node.text().collect::<String>());
        let variant = variant_from_label(&label);
        let display = label.trim().to_string();

        entries.push(DownloadEntry {
            label: if display.is_empty() {
                host.clone()
            } else {
                display
            },
            variant,
            host,
            platform: current_platform.clone(),
            go_link,
        });
    }
    entries
}

/// Host slug from the `d-<host>` class on a download anchor.
fn host_from_classes(node: ElementRef) -> String {
    node.value()
        .attr("class")
        .unwrap_or_default()
        .split_whitespace()
        .find_map(|cls| cls.strip_prefix("d-"))
        .unwrap_or_default()
        .to_string()
}

/// Trailing `(Parenthesized)` suffix → variant, else None.
fn variant_from_label(label: &str) -> Option<String> {
    let trimmed = label.trim();
    if !(trimmed.ends_with(')') && trimmed.contains('(')) {
        return None;
    }
    let start = trimmed.rfind('(')?;
    let inner = trimmed[start + 1..trimmed.len() - 1].trim();
    if inner.is_empty() {
        None
    } else {
        Some(inner.to_string())
    }
}

/// `<link rel="canonical" href=".../game/<slug>/">` → slug (or None).
fn canonical_slug(document: &Html) -> Option<String> {
    let sel = Selector::parse("link[rel='canonical'][href]").expect("valid canonical selector");
    document
        .select(&sel)
        .next()
        .and_then(|el| el.value().attr("href"))
        .and_then(slug_from_url)
}

/// Last non-empty path segment of a game URL.
fn slug_from_url(url: &str) -> Option<String> {
    let trimmed = url.trim_end_matches('/');
    let slug = trimmed.rsplit('/').next()?;
    if slug.is_empty() || slug.contains('.') || slug == "game" {
        None
    } else {
        Some(slug.to_string())
    }
}

/// Meta content for a given name/property (`description`, `og:url`).
fn meta(document: &Html, key: &str) -> Option<String> {
    let sel = Selector::parse(&format!("meta[name='{}'], meta[property='{}']", key, key))
        .expect("valid meta selector");
    document
        .select(&sel)
        .next()
        .and_then(|el| el.value().attr("content"))
        .map(clean_text)
        .filter(|s| !s.is_empty())
}

fn first_text(document: &Html, selector: &str) -> Option<String> {
    let sel = Selector::parse(selector).ok()?;
    document
        .select(&sel)
        .next()
        .map(|el| clean_text(&el.text().collect::<String>()))
}

fn clean_text(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../../tests/fixtures/html/lz_game.html");

    fn parsed() -> Game {
        parse_game(FIXTURE)
    }

    #[test]
    fn parses_core_identity() {
        let game = parsed();
        assert_eq!(game.slug, "treasure-of-nadia");
        assert_eq!(game.post_id, Some(18212));
        assert_eq!(game.title, "Treasure of Nadia [Finished] - Version: 1.0117");
        assert_eq!(game.developer.as_deref(), Some("NLT Media"));
        assert_eq!(game.current_version.as_deref(), Some("1.0117 (Finished)"));
    }

    #[test]
    fn parses_header_details() {
        let game = parsed();
        assert_eq!(game.engine.as_deref(), Some("RPGM"));
        assert_eq!(game.size_label.as_deref(), Some("7.51 GB"));
        assert_eq!(game.censorship.as_deref(), Some("Uncensored"));
    }

    #[test]
    fn parses_genres_and_slug_body() {
        let game = parsed();
        assert!(game.genres.iter().any(|g| g == "3DCG"));
        assert!(game.genres.iter().any(|g| g == "Adventure"));
        assert!(game
            .description
            .as_deref()
            .unwrap_or_default()
            .contains("Treasure of Nadia"));
    }

    #[test]
    fn parses_screenshots_full_res() {
        let game = parsed();
        assert!(!game.screenshots.is_empty());
        assert!(game
            .screenshots
            .iter()
            .all(|s| s.contains("wp-content/uploads") && s.contains(".jpeg")));
        assert_eq!(game.screenshots.len(), 9);
    }

    #[test]
    fn parses_versions_with_latest_flag() {
        let game = parsed();
        assert!(!game.versions.is_empty());
        assert!(game.versions[0].is_latest);
        assert!(game.versions[0].label.contains("v1.0117"));
        assert!(game.versions.len() >= 2, "has older `?ver=` options");
    }

    #[test]
    fn official_tab_yields_windows_compressed_entries() {
        let game = parsed();
        let official: Vec<_> = game
            .versions
            .first()
            .map(|v| v.official.clone())
            .unwrap_or_default();
        assert!(!official.is_empty(), "official tab has rows");
        let fileknot = official
            .iter()
            .find(|e| e.host == "fileknot")
            .expect("fileknot row present");
        assert_eq!(fileknot.platform.as_deref(), Some("pc"));
        assert!(fileknot.go_link.contains("lewdzone.com/go/#t=v1."));

        let compressed = official
            .iter()
            .find(|e| e.variant.as_deref() == Some("Compressed"))
            .expect("compressed row present");
        assert_eq!(compressed.host, "fileknot");
        assert_eq!(compressed.platform.as_deref(), Some("pc"));
    }

    #[test]
    fn community_tab_has_distinct_hosts() {
        let game = parsed();
        let community = &game.versions[0].community;
        assert!(!community.is_empty());
        let hosts: Vec<_> = community.iter().map(|e| e.host.as_str()).collect();
        assert!(hosts.iter().any(|h| *h == "gofile" || *h == "mega"));
        assert!(hosts.contains(&"uploadhaven"));
    }

    #[test]
    fn download_entries_flattened_across_tabs() {
        let game = parsed();
        assert!(!game.download_entries.is_empty());
        assert!(game.download_entries.iter().any(|e| e.host == "fileknot"));
        assert!(game.download_entries.iter().any(|e| e.host == "mega"));
    }

    #[test]
    fn platform_tracking_covers_android_and_mac() {
        let game = parsed();
        let platforms: Vec<_> = game.platforms.clone();
        assert!(platforms.contains(&"pc".to_string()));
        assert!(platforms.contains(&"android".to_string()));
        assert!(platforms.contains(&"mac".to_string()));
    }

    #[test]
    fn variant_labels_parse() {
        assert_eq!(
            variant_from_label("Fileknot (Compressed)"),
            Some("Compressed".to_string())
        );
        assert_eq!(
            variant_from_label("Transfaze  (Part 1 Compressed)"),
            Some("Part 1 Compressed".to_string())
        );
        assert_eq!(variant_from_label("Mega "), None);
        assert_eq!(variant_from_label(""), None);
    }

    #[test]
    fn parses_cover_url_from_fixture() {
        let cover = parse_cover_url(FIXTURE);
        assert_eq!(
            cover.as_deref(),
            Some("https://lewdzone.com/wp-content/uploads/2019/10/Treasure-of-Nadi-Adult-XXX-Game-Cover-329x196.jpeg")
        );
    }
}
