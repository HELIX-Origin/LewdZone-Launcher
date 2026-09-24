//! Folder organizer — canonical game-folder layout and filename
//! sanitization (dm.md, folder-organizer.md; ADR-0005).
//!
//! Download layout: `<DownloadRoot>/Games/<Title>/<Title> - <Version> -
//! <Platform>[- <Variant>].<ext>`. Multi-part downloads merge into
//! `<Title>/_parts/`. Installed apps are extracted to `<LzappsRoot>/<slug>/`
//! and tracked with an itch.io-style `app.json` manifest.
//!
//! Also owns the **download root** and **lzapps root** resolution now that the
//! download-manager layer is gone: in-app downloads land under `download-root`
//! (defaulting to the per-OS `downloads/` folder), and installed apps land
//! under `library-root/lzapps` (or the per-OS `lzapps/` folder).

use std::path::{Component, Path, PathBuf};

use crate::core::{Context, Error};

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

/// Installed app folder: `<lzapps_root>/<slug>/`. One folder per game slug;
/// updates replace the contents of this folder.
pub fn install_dir(lzapps_root: &Path, slug: &str) -> PathBuf {
    lzapps_root.join(sanitize_segment(slug))
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

/// Resolve where in-app downloads land: `download-root` when set, then
/// `games_dir/downloads` (or `games_dir`), `library-root/downloads`, then the per-OS `downloads/` folder.
pub fn download_root(ctx: &Context) -> Result<PathBuf, Error> {
    let settings = crate::core::settings::Settings::load(&ctx.config_path)?;
    if let Some(root) = settings.download_root.filter(|r| !r.trim().is_empty()) {
        return Ok(PathBuf::from(root));
    }
    if let Some(gdir) = settings.games_dir.filter(|r| !r.trim().is_empty()) {
        let p = PathBuf::from(gdir);
        if p.join("downloads").is_dir() {
            return Ok(p.join("downloads"));
        }
        return Ok(p.join("downloads"));
    }
    if let Some(root) = settings.library_root.filter(|r| !r.trim().is_empty()) {
        let p = PathBuf::from(root);
        if p.join("downloads").is_dir() {
            return Ok(p.join("downloads"));
        }
        return Ok(p.join("downloads"));
    }
    crate::core::paths::downloads_dir()
        .ok_or_else(|| Error::Runtime("cannot resolve download root (no app data dir)".into()))
}

/// Build the canonical download target for an archive directly inside downloads:
/// `downloads/<archive>` (or `<root>/<archive>` if root is already the downloads dir).
pub fn archive_download_target(root: &Path, archive_name: &str) -> PathBuf {
    let dir = if root
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|s| s.eq_ignore_ascii_case("downloads"))
    {
        root.to_path_buf()
    } else {
        root.join("downloads")
    };
    non_colliding(&dir.join(archive_name))
}

pub fn engine_download_target(root: &Path, _engine: Option<&str>, archive_name: &str) -> PathBuf {
    archive_download_target(root, archive_name)
}

/// Build the canonical installed app folder directly inside installed:
/// `installed/<slug>/` (or `<root>/<slug>/` if root is already the installed dir).
pub fn engine_install_dir(installed_root: &Path, _engine: Option<&str>, slug: &str) -> PathBuf {
    let dir = if installed_root
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|s| s.eq_ignore_ascii_case("installed"))
    {
        installed_root.to_path_buf()
    } else {
        installed_root.join("installed")
    };
    install_dir(&dir, slug)
}

/// Normalize an engine name to a lowercase directory-friendly folder name.
pub fn engine_to_folder(engine: Option<&str>) -> String {
    let Some(raw) = engine else {
        return "other".to_string();
    };
    let lower = raw.trim().to_lowercase();
    if lower.contains("renpy") || lower.contains("ren'py") {
        "renpy".to_string()
    } else if lower.contains("rpg") {
        "rpgm".to_string()
    } else if lower.contains("unity") {
        "unity".to_string()
    } else if lower.contains("unreal") {
        "unreal".to_string()
    } else if lower.contains("html") || lower.contains("nw.js") || lower.contains("nwjs") {
        "html".to_string()
    } else if lower.contains("godot") {
        "godot".to_string()
    } else if lower.contains("flash") {
        "flash".to_string()
    } else if lower.contains("wolf") {
        "wolfrpg".to_string()
    } else if lower.contains("qsp") {
        "qsp".to_string()
    } else if lower.contains("twine") {
        "twine".to_string()
    } else if lower.contains("tyrano") {
        "tyrano".to_string()
    } else if lower.contains("rags") {
        "rags".to_string()
    } else if lower.contains("tads") {
        "tads".to_string()
    } else if lower.contains("java") {
        "java".to_string()
    } else if lower.contains("python") {
        "python".to_string()
    } else if lower.contains("webgl") {
        "webgl".to_string()
    } else {
        let s: String = lower
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect();
        if s.is_empty() {
            "other".to_string()
        } else {
            s
        }
    }
}

/// Resolve where installed / extracted games live: `games_dir/installed` or `games_dir`,
/// `library-root/installed`, `library-root/lzapps`, or the per-OS `lzapps/` folder.
pub fn installed_root(ctx: &Context) -> Result<PathBuf, Error> {
    let settings = crate::core::settings::Settings::load(&ctx.config_path)?;
    if let Some(gdir) = settings.games_dir.filter(|r| !r.trim().is_empty()) {
        let p = PathBuf::from(&gdir);
        if p.join("installed").is_dir() {
            return Ok(p.join("installed"));
        }
        return Ok(p);
    }
    if let Some(root) = settings.library_root.filter(|r| !r.trim().is_empty()) {
        let p = PathBuf::from(&root);
        if p.join("installed").is_dir() {
            return Ok(p.join("installed"));
        }
        if p.join("lzapps").is_dir() {
            return Ok(p.join("lzapps"));
        }
        return Ok(p.join("installed"));
    }
    if let Some(lz) = crate::core::paths::lzapps_dir() {
        if let Some(parent) = lz.parent() {
            let inst = parent.join("installed");
            if inst.exists() {
                return Ok(inst);
            }
        }
    }
    crate::core::paths::lzapps_dir()
        .ok_or_else(|| Error::Runtime("cannot resolve installed root (no app data dir)".into()))
}

/// Resolve where installed / extracted apps land: `library-root/installed` (if exists),
/// `library-root/lzapps`, otherwise the per-OS `lzapps/` folder.
pub fn lzapps_root(ctx: &Context) -> Result<PathBuf, Error> {
    let settings = crate::core::settings::Settings::load(&ctx.config_path)?;
    if let Some(root) = settings.library_root.filter(|r| !r.trim().is_empty()) {
        let p = PathBuf::from(&root);
        if p.join("installed").is_dir() {
            return Ok(p.join("installed"));
        }
        return Ok(p.join("lzapps"));
    }
    crate::core::paths::lzapps_dir()
        .ok_or_else(|| Error::Runtime("cannot resolve lzapps root (no app data dir)".into()))
}

/// Supported standard game engines for directory organization.
pub const STANDARD_ENGINES: &[&str] = &[
    "renpy", "rpgm", "unity", "unreal", "html", "godot", "flash", "wolfrpg", "qsp", "twine",
    "tyrano", "rags", "tads", "java", "python", "webgl", "other",
];

/// Automatically create the canonical `downloads/` and `installed/` folders under `root`.
pub fn initialize_library_structure(root: &Path) -> Result<(), Error> {
    if !root.exists() {
        std::fs::create_dir_all(root)?;
    }
    let dl_root = root.join("downloads");
    let inst_root = root.join("installed");

    std::fs::create_dir_all(&dl_root)?;
    std::fs::create_dir_all(&inst_root)?;
    Ok(())
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
        let root = Path::new("/tmp/lewdzone-test/games");
        assert_eq!(
            game_dir(root, "Treasure of Nadia"),
            Path::new("/tmp/lewdzone-test/games/Games/Treasure of Nadia")
        );
        assert_eq!(
            parts_dir(root, "Treasure of Nadia"),
            Path::new("/tmp/lewdzone-test/games/Games/Treasure of Nadia/_parts")
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
        let root = Path::new("test-root");
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
        let root = Path::new("test-root");
        assert!(within_root(root, &root.join("Games/T/ok.zip")));
        assert!(!within_root(root, &root.join("..")).to_owned_assert_false());
        assert!(!within_root(root, &root.join("../evil.zip")));
        assert!(!within_root(root, Path::new("foreign/x.zip")));
    }

    trait ToOwnedAssertFalse {
        fn to_owned_assert_false(&self) -> bool;
    }
    impl ToOwnedAssertFalse for bool {
        fn to_owned_assert_false(&self) -> bool {
            *self
        }
    }

    #[test]
    fn install_dir_is_slug_based() {
        let root = Path::new("test-lzapps-root");
        assert_eq!(
            install_dir(root, "treasure-of-nadia"),
            Path::new("test-lzapps-root").join("treasure-of-nadia")
        );
        assert_eq!(
            install_dir(root, "bad/slug?"),
            Path::new("test-lzapps-root").join("badslug")
        );
    }

    fn tmp_ctx(tag: &str) -> Context {
        let tmp = std::env::temp_dir().join(format!(
            "lewdzone-folder-test-{}-{}",
            tag,
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let db = tmp.join(format!("{tag}.db"));
        let cfg = tmp.join(format!("{tag}.json"));
        std::fs::write(&cfg, b"{}").unwrap();
        Context::new(db, cfg)
    }

    #[test]
    fn download_root_prefers_download_root_setting() {
        let ctx = tmp_ctx("dl-root");
        std::fs::write(&ctx.config_path, br#"{"download_root":"custom-raw-root"}"#).unwrap();
        assert_eq!(download_root(&ctx).unwrap(), Path::new("custom-raw-root"));
    }

    #[test]
    fn download_root_falls_back_to_library_root_downloads() {
        let ctx = tmp_ctx("lib-dl");
        std::fs::write(
            &ctx.config_path,
            br#"{"library_root":"custom-library-root"}"#,
        )
        .unwrap();
        assert_eq!(
            download_root(&ctx).unwrap(),
            Path::new("custom-library-root").join("downloads")
        );
    }

    #[test]
    fn lzapps_root_prefers_library_root_lzapps() {
        let ctx = tmp_ctx("lib-lz");
        std::fs::write(
            &ctx.config_path,
            br#"{"library_root":"custom-library-root"}"#,
        )
        .unwrap();
        assert_eq!(
            lzapps_root(&ctx).unwrap(),
            Path::new("custom-library-root").join("lzapps")
        );
    }

    #[test]
    fn roots_default_to_per_os_folders() {
        let ctx = tmp_ctx("roots-default");
        let dl = download_root(&ctx).unwrap();
        let lz = lzapps_root(&ctx).unwrap();
        assert!(dl.ends_with("downloads"), "downloads default: {dl:?}");
        assert!(lz.ends_with("lzapps"), "lzapps default: {lz:?}");
        assert!(dl.is_absolute());
        assert!(lz.is_absolute());
    }

    #[test]
    fn initialize_library_structure_creates_flat_directories() {
        let tmp = std::env::temp_dir().join(format!(
            "lz-lib-struct-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        initialize_library_structure(&tmp).expect("initializes library structure");

        assert!(tmp.join("downloads").is_dir());
        assert!(tmp.join("installed").is_dir());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn engine_to_folder_normalizes_correctly() {
        assert_eq!(engine_to_folder(Some("Ren'Py")), "renpy");
        assert_eq!(engine_to_folder(Some("RenPy")), "renpy");
        assert_eq!(engine_to_folder(Some("RPG Maker MV")), "rpgm");
        assert_eq!(engine_to_folder(Some("Unity")), "unity");
        assert_eq!(engine_to_folder(Some("Unreal Engine 5")), "unreal");
        assert_eq!(engine_to_folder(Some("HTML/NW.js")), "html");
        assert_eq!(engine_to_folder(Some("Godot")), "godot");
        assert_eq!(engine_to_folder(Some("Flash")), "flash");
        assert_eq!(engine_to_folder(Some("QSP")), "qsp");
        assert_eq!(engine_to_folder(Some("Twine")), "twine");
        assert_eq!(engine_to_folder(Some("TyranoBuilder")), "tyrano");
        assert_eq!(engine_to_folder(Some("RAGS")), "rags");
        assert_eq!(engine_to_folder(Some("TADS")), "tads");
        assert_eq!(engine_to_folder(Some("Java")), "java");
        assert_eq!(engine_to_folder(Some("Python")), "python");
        assert_eq!(engine_to_folder(Some("WebGL")), "webgl");
        assert_eq!(engine_to_folder(None), "other");
    }

    #[test]
    fn archive_download_target_organizes_directly_under_downloads() {
        let root = Path::new("G:/LewdZone/downloads");
        let target = archive_download_target(root, "game.zip");
        assert_eq!(target, PathBuf::from("G:/LewdZone/downloads/game.zip"));

        let lib_root = Path::new("G:/LewdZone");
        let target2 = archive_download_target(lib_root, "game2.zip");
        assert_eq!(target2, PathBuf::from("G:/LewdZone/downloads/game2.zip"));
    }

    #[test]
    fn engine_install_dir_organizes_directly_under_installed() {
        let root = Path::new("G:/LewdZone/installed");
        let target = engine_install_dir(root, None, "my-slug");
        assert_eq!(target, PathBuf::from("G:/LewdZone/installed/my-slug"));

        let lib_root = Path::new("G:/LewdZone");
        let target2 = engine_install_dir(lib_root, None, "rpg-game");
        assert_eq!(target2, PathBuf::from("G:/LewdZone/installed/rpg-game"));
    }
}
