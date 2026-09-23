//! Canonical domain models — authoritative shape for every family (owner:
//! `database/schema-designer`). Scrapers, resolvers, repositories and the
//! Tauri commands all map to these structs. They derive `Clone` + `Serialize`
//! so they survive SQLite round-trips and cross the webview boundary.
//!
//! Unknown/absent sections are always `Option<T>` / `Vec` — parsers never
//! crash on a missing WordPress section (Rule: structural unknowns are
//! optional).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A game tile in a listing page (archive, genre, search results).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameCard {
    /// URL slug, e.g. `wild-life` (the stable public id).
    pub slug: String,
    /// Display title.
    pub title: String,
    /// WordPress post id when known (from the game page / `?p=` redirect).
    pub post_id: Option<i64>,
    /// Cover thumbnail URL (`img.preview[data-src]`).
    pub thumb_url: Option<String>,
    /// Normalized platform slugs: `pc`, `android`, `linux`, `mac`.
    pub platforms: Vec<String>,
    /// Engine when listed, e.g. "Unity", "Ren'Py".
    pub engine: Option<String>,
    /// Dev state: Ongoing / Finished / Abandoned.
    pub state: Option<String>,
    /// The `version-tag` string, e.g. "v2026-06-15 Full".
    pub version_tag: Option<String>,
    /// Developer / studio when shown ("by X").
    pub developer: Option<String>,
    /// Description snippet (`div.content`).
    pub description: Option<String>,
    /// Genre display names (tag text).
    pub genres: Vec<String>,
    /// Genre slugs (from `/game-genre/<slug>/` hrefs).
    pub genre_slugs: Vec<String>,
    /// Last update date text, e.g. "June 18, 2026".
    pub updated_at: Option<String>,
    /// Human view count label, e.g. "962K".
    pub views: Option<String>,
}

/// One entry in the `/game-genres/` tag cloud.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Genre {
    /// Display label, e.g. "2D Game".
    pub label: String,
    /// URL slug, e.g. `2d-game`.
    pub slug: String,
    /// Game count shown by the site, when disclosed.
    pub count: Option<u32>,
}

/// Filters for the archive listing, mirroring the site's GET params 1:1
/// (`q`, `platform`, `engine`, `state`, `sort`, repeated `tags[]` /
/// `tags-exclude[]`). The Store view holds one of these and sends it to
/// `catalog_page`; the CLI exposes the same surface through `sync --filter`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ArchiveFilter {
    /// Free-text `q` (site search box).
    pub q: Option<String>,
    /// Platform select: `All`, `PC`, `Mac`, `Linux`, `Android`.
    pub platform: Option<String>,
    /// Engine select: `RenPy`, `RPG Maker`, `Unity`, `Unreal Engine`,
    /// `HTML`, `Flash`, `Wolf RPG`, `Other`.
    pub engine: Option<String>,
    /// Dev state select: `Finished`, `Ongoing`, `Abandoned`, `Onhold`,
    /// `Demo`.
    pub state: Option<String>,
    /// Sort select: `Last Update`, `Popularity`, `New to Old`, `Old to
    /// New`, `Rating`.
    pub sort: Option<String>,
    /// Included `tags[]` slugs (genre taxonomy, e.g. `2d-game`, `rpg`).
    pub include_tags: Vec<String>,
    /// Excluded `tags-exclude[]` slugs.
    pub exclude_tags: Vec<String>,
}

/// Page-level metadata for a listing (archive / genre / search).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ArchiveMeta {
    /// 1-based current page.
    pub page: u32,
    /// Total pages from `.wp-pagenavi .pages`, when disclosed.
    pub total_pages: Option<u32>,
    /// Applied filters, e.g. `{ "sort": "popularity", "platform": "pc" }`.
    pub applied: BTreeMap<String, String>,
}

/// One game's full profile (from `/game/<slug>/`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Game {
    pub slug: String,
    pub post_id: Option<i64>,
    pub title: String,
    pub developer: Option<String>,
    pub current_version: Option<String>,
    pub engine: Option<String>,
    /// Normalized platform slugs: `pc`, `android`, `linux`, `mac`.
    pub platforms: Vec<String>,
    /// Genre slugs, e.g. `3dcg`, `incest`, `harem`.
    pub genres: Vec<String>,
    /// Human size label, e.g. "7.51 GB" (parse via `parse_size` when numeric).
    pub size_label: Option<String>,
    pub censorship: Option<String>,
    pub screenshots: Vec<String>,
    pub description: Option<String>,
    /// All versions with their download tabs, newest first.
    pub versions: Vec<Version>,
    /// All known download links (flat view across versions/tabs).
    pub download_entries: Vec<DownloadEntry>,
}

/// A release version and its official / community download tabs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Version {
    /// Display label, e.g. `v1.0117` or `v2026-06-15 Full`.
    pub label: String,
    /// Site-flagged latest release.
    pub is_latest: bool,
    pub official: Vec<DownloadEntry>,
    pub community: Vec<DownloadEntry>,
}

/// One download row on a game page (`downloadLink d-<host>`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DownloadEntry {
    /// Human label of the row, e.g. "PC (Compressed)".
    pub label: String,
    /// Variant suffix when present, e.g. "Part 1", "Compressed".
    pub variant: Option<String>,
    /// Host slug (`d-<host>` class / go payload), e.g. "fileknot".
    pub host: String,
    /// Normalized platform for this row (`pc`, `android`, `linux`, `mac`).
    pub platform: Option<String>,
    /// Full go-link href, e.g. `https://lewdzone.com/go/#t=v1.<p>.<s>`.
    pub go_link: String,
}

/// Normalize a platform family icon / query token to the canonical slug
/// (`pc`, `android`, `linux`, `mac`). Unknown tokens stay as-is.
pub fn normalize_platform(raw: &str) -> String {
    match raw.to_ascii_lowercase().trim() {
        "fa-windows" | "windows" | "pc" | "win" => "pc".to_string(),
        "fa-android" | "android" => "android".to_string(),
        "fa-apple" | "apple" | "ios" | "mac" | "macos" | "osx" => "mac".to_string(),
        "fa-linux" | "linux" => "linux".to_string(),
        other => other.to_string(),
    }
}

/// Best-effort parse of a human size like "7.51 GB" / "850 MB" to bytes.
pub fn parse_size(label: &str) -> Option<u64> {
    let label = label.trim();
    let num: f64 = label
        .split_whitespace()
        .next()?
        .replace(',', "")
        .parse()
        .ok()?;
    let unit = label.to_ascii_lowercase();
    let mult = if unit.contains("tb") {
        1024u64.pow(4)
    } else if unit.contains("gb") {
        1024u64.pow(3)
    } else if unit.contains("mb") {
        1024u64.pow(2)
    } else if unit.contains("kb") {
        1024
    } else {
        return None;
    };
    Some((num * mult as f64) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_platform_families() {
        assert_eq!(normalize_platform("fa-windows"), "pc");
        assert_eq!(normalize_platform("WINDOWS"), "pc");
        assert_eq!(normalize_platform("mac"), "mac");
        assert_eq!(normalize_platform("fa-android"), "android");
        assert_eq!(normalize_platform("fa-linux"), "linux");
        assert_eq!(normalize_platform("atari"), "atari");
    }

    #[test]
    fn parses_human_size_labels() {
        assert_eq!(parse_size("850 MB"), Some(891_289_600));
        assert_eq!(
            parse_size("1.5 GB"),
            Some((1.5_f64 * 1024.0_f64.powi(3)) as u64)
        );
        assert!(parse_size("unknown").is_none());
        assert!(parse_size("").is_none());
    }

    #[test]
    fn cards_round_trip_through_json() {
        let card = GameCard {
            slug: "wild-life".into(),
            title: "Wild Life".into(),
            post_id: None,
            thumb_url: Some("https://lewdzone.com/cover.jpg".into()),
            platforms: vec!["pc".into(), "mac".into()],
            engine: Some("Unreal Engine".into()),
            state: Some("Ongoing".into()),
            version_tag: Some("v2026-06-15 Full".into()),
            developer: Some("Adeptus Steve".into()),
            description: Some("A mad universe...".into()),
            genres: vec!["3D Game".into()],
            genre_slugs: vec!["3d-games".into()],
            updated_at: Some("June 18, 2026".into()),
            views: Some("962K".into()),
        };
        let cloned =
            serde_json::from_str::<GameCard>(&serde_json::to_string(&card).unwrap()).unwrap();
        assert_eq!(cloned, card);
    }
}
