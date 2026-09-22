//! Folder organizer — canonical game-folder layout and filename
//! sanitization (dm.md, folder-organizer.md; ADR-0005).
//!
//! Layout: `<DownloadRoot>/Games/<Title>/<Title> - <Version> - <Platform>
//! [- <Variant>].<ext>`. Multi-part downloads merge into `<Title>/_parts/`.
//! All name logic here is pure and unit-tested; file moves belong to the
//! download pipeline (todo #6).

use std::path::{Component, Path, PathBuf};

/// Charset never allowed in a file or folder name.
const FORBIDDEN: &[char] = &['\\', '/', ':', '*', '?', '"', '<', '>', '|'];

/// Sanitize a single path segment per Rule 02: strip the forbidden charset,
/// collapse whitespace runs to one space, and trim leading/trailing dots
/// and spaces.
pub fn sanitize_segment(raw: &str) -> String {
    let cleaned: String = raw.chars().filter(|c| !FORBIDDEN.contains(c)).collect();
    let collapsed: String = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed
        .trim_matches(|c: char| c == '.' || c == ' ')
        .to_string()
}

/// Build `<Title> - <Version> - <Platform>[- <Variant>]` basename (no ext).
pub fn safe_basename(title: &str, version: &str, platform: &str, variant: Option<&str>) -> String {
    let base = format!(
        "{} - {} - {}",
        sanitize_segment(title),
        sanitize_segment(version),
        sanitize_segment(platform)
    );
    match variant.map(sanitize_segment).filter(|v| !v.is_empty()) {
        Some(v) => format!("{base} - {v}"),
        None => base,
    }
}

/// Remove any `.`-suffixed extension from the end of `raw` (returns empty for
/// hidden files or names with no extension).
pub fn file_stem(raw: &str) -> &str {
    match raw.rfind('.') {
        Some(idx) if idx > 0 => &raw[..idx],
        _ => raw,
    }
}

/// The user-visible extension segment, e.g. `zip` for `file.zip`; empty when
/// there is no extension.
pub fn file_ext(raw: &str) -> &str {
    match raw.rfind('.') {
        Some(idx) if idx > 0 => &raw[idx + 1..],
        _ => "",
    }
}

/// Compute the final game folder path `<root>/Games/<Title>`. Multi-part
/// parts get an extra `_parts/` child (via [`parts_dir`]).
pub fn game_dir(root: &Path, title: &str) -> PathBuf {
    root.join("Games").join(sanitize_segment(title))
}

/// `_parts/` subfolder inside a game dir for multi-part downloads.
pub fn parts_dir(root: &Path, title: &str) -> PathBuf {
    game_dir(root, title).join("_parts")
}

/// Pick a non-colliding filename for `target` by appending ` (N)` before the
/// extension (Rule 07: never overwrite an existing file).
pub fn non_colliding(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }
    let parent = path.parent().unwrap_or(Path::new(""));
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let stem = file_stem(name);
    let ext = file_ext(name);
    let suffix = if ext.is_empty() {
        String::new()
    } else {
        format!(".{ext}")
    };
    for i in 1..=10_000_u32 {
        let cand = parent.join(format!("{stem} ({i}){suffix}"));
        if !cand.exists() {
            return cand;
        }
    }
    path.to_path_buf()
}

/// Build the canonical download target for a game file. `/torrent` and
/// magnet payloads land in `_parts/`; direct files go in the game dir.
pub fn download_target(
    root: &Path,
    title: &str,
    version: &str,
    platform: &str,
    variant: Option<&str>,
    original_name: &str,
    is_multi_part: bool,
) -> PathBuf {
    let ext = file_ext(original_name);
    let ext = if ext.is_empty() {
        String::new()
    } else {
        format!(".{ext}")
    };
    let base = safe_basename(title, version, platform, variant);
    let dir = if is_multi_part {
        parts_dir(root, title)
    } else {
        game_dir(root, title)
    };
    non_colliding(&dir.join(format!("{base}{ext}")))
}

/// Assert `out` never escapes the download root (no `..`, no rooted path).
pub fn within_root(root: &Path, out: &Path) -> bool {
    let Ok(rel) = out.strip_prefix(root) else {
        return false;
    };
    !rel.components().any(|c| {
        matches!(
            c,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_strips_forbidden_chars() {
        assert_eq!(
            sanitize_segment(r#"Final: <Cut> "v2" | Remix?"#),
            "Final Cut v2 Remix"
        );
    }

    #[test]
    fn sanitize_collapses_spaces_and_trims_dots() {
        assert_eq!(sanitize_segment("  Lust    for Mars.  "), "Lust for Mars");
        assert_eq!(sanitize_segment("...."), "");
    }

    #[test]
    fn basename_composes_with_variant() {
        assert_eq!(
            safe_basename("Treasure of Nadia", "1.0117", "pc", None),
            "Treasure of Nadia - 1.0117 - pc"
        );
        assert_eq!(
            safe_basename("A/B Game", "v2", "pc", Some("Part 1")),
            "AB Game - v2 - pc - Part 1"
        );
    }

    #[test]
    fn stem_and_ext_split() {
        assert_eq!(file_stem("file.zip"), "file");
        assert_eq!(file_ext("file.zip"), "zip");
        assert_eq!(file_stem("noext"), "noext");
        assert_eq!(file_ext("noext"), "");
    }

    #[test]
    fn game_dir_layout_matches_adr() {
        let root = Path::new("D:/Games");
        assert_eq!(
            game_dir(root, "Treasure of Nadia"),
            Path::new("D:/Games/Games/Treasure of Nadia")
        );
        assert_eq!(
            parts_dir(root, "Treasure of Nadia"),
            Path::new("D:/Games/Games/Treasure of Nadia/_parts")
        );
    }

    #[test]
    fn non_colliding_returns_first_free_name() {
        let tmp = std::env::temp_dir().join(format!("lewdzone-fold-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        let target = tmp.join("game - v1 - pc.zip");
        std::fs::write(&target, b"x").unwrap();
        let next = non_colliding(&target);
        assert_eq!(
            next.file_name().and_then(|n| n.to_str()),
            Some("game - v1 - pc (1).zip")
        );
        std::fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn multi_part_target_lands_in_parts_dir() {
        let root = Path::new("D:/Games");
        let t = download_target(
            root,
            "Wild Life",
            "v2026",
            "pc",
            Some("Part 1 Compressed"),
            "wildlife.part1.rar",
            true,
        );
        assert!(t.starts_with(root.join("Games/Wild Life/_parts")));
        assert!(t.to_string_lossy().contains("v2026"));
    }

    #[test]
    fn within_root_rejects_traversal() {
        let root = Path::new("D:/Games");
        assert!(within_root(root, &root.join("Games/T/ok.zip")));
        assert!(!within_root(root, &root.join("..")).to_owned_assert_false());
        assert!(!within_root(root, &root.join("../evil.zip")));
        assert!(!within_root(root, Path::new("C:/x.zip")));
    }

    trait ToOwnedAssertFalse {
        fn to_owned_assert_false(&self) -> bool;
    }
    impl ToOwnedAssertFalse for bool {
        fn to_owned_assert_false(&self) -> bool {
            *self
        }
    }
}
