//! `download` — resolve a token and hand the URL to a download manager
//! (ADR-0002, dispatch-builder + launch-download skill).
//!
//! The pipeline is: pick a `Game` page → find the `DownloadEntry` matching
//! (version, platform, tab) → resolve its go-link via the start→reveal API →
//! dispatch the real URL to the active manager with a canonical folder
//! basename (folder-organizer). The game fetch and the resolver are injectable
//! so every path is covered offline with fixtures (Rule 11).

use std::path::PathBuf;

use crate::core::{Context, Error};
use crate::dm;
use crate::dm::folder;
use crate::resolver;
use crate::scraper;

use crate::core::models::{DownloadEntry, Game};

/// A fully dispatched download job (what `run` prints back).
#[derive(Debug, Clone, serde::Serialize)]
pub struct Job {
    pub game: String,
    pub version: String,
    pub platform: String,
    pub tab: String,
    pub manager: String,
    pub url: String,
    pub target: Option<PathBuf>,
}

/// The user-facing selection of what to fetch and grab from a game page.
#[derive(Debug, Clone, Default)]
pub struct Select<'a> {
    pub game: &'a str,
    pub version: &'a str,
    pub platform: &'a str,
    pub tab: &'a str,
}

/// Pick the version object matching `version` (or the site-flagged latest),
/// then collect every official/community entry for `platform`.
///
/// Returns a user-facing error naming what was actually available so the
/// mismatch is diagnosable without guessing (dispatch-builder rule).
pub fn pick_entries<'a>(
    game: &'a Game,
    version: &str,
    platform: &str,
    tab: &str,
) -> Result<Vec<&'a DownloadEntry>, Error> {
    let ver = game
        .versions
        .iter()
        .find(|v| {
            version != "latest"
                && (v.label.eq_ignore_ascii_case(&format!("v{version}"))
                    || v.label.eq_ignore_ascii_case(version))
        })
        .or_else(|| {
            if version == "latest" {
                game.versions
                    .iter()
                    .find(|v| v.is_latest)
                    .or_else(|| game.versions.first())
            } else {
                None
            }
        })
        .ok_or_else(|| {
            let have: Vec<&str> = game.versions.iter().map(|v| v.label.as_str()).collect();
            Error::Usage(format!(
                "game '{}' has no version '{version}' (available: {})",
                game.slug,
                have.join(", ")
            ))
        })?;

    let pool = match tab {
        "official" => &ver.official,
        "community" => &ver.community,
        other => {
            return Err(Error::Usage(format!(
                "unknown tab '{other}' (expected 'official' or 'community')"
            )))
        }
    };
    let norm = crate::core::models::normalize_platform(platform);
    let entries: Vec<&DownloadEntry> = pool
        .iter()
        .filter(|e| e.platform.as_deref().map(|p| p == norm).unwrap_or(false))
        .collect();
    if entries.is_empty() {
        let have: Vec<&str> = pool
            .iter()
            .filter_map(|e| e.platform.as_deref())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        return Err(Error::Usage(format!(
            "no {tab} entry for platform '{norm}' in {} (available: {})",
            ver.label,
            if have.is_empty() {
                "none".to_string()
            } else {
                have.join(", ")
            }
        )));
    }
    Ok(entries)
}

/// Resolve the go-link for one entry and build a canonical folder basename.
pub fn job_for(
    game: &Game,
    entry: &DownloadEntry,
    manager: &str,
    root: &std::path::Path,
    resolve: &mut dyn FnMut(&str) -> Result<resolver::ResolvedUrl, Error>,
) -> Result<Job, Error> {
    crate::resolver::validate_host(&entry.host)?;
    let resolved = resolve(&entry.go_link)?;
    let variant = entry.variant.as_deref();
    let orig: &str = resolved
        .url
        .rsplit('/')
        .next()
        .unwrap_or_else(|| &resolved.url);
    let platform = entry.platform.as_deref().unwrap_or("pc").to_string();
    let base = folder::safe_basename(&game.title, &entry_label(entry), &platform, variant);
    let ext = folder::file_ext(orig);
    let target_name = format!("{base}.{ext}");
    let final_path = folder::download_target(
        root,
        &game.title,
        &entry_label(entry),
        &platform,
        variant,
        &target_name,
        false,
    );
    Ok(Job {
        game: game.slug.clone(),
        version: entry_label(entry),
        platform,
        tab: entry.host.clone(),
        manager: manager.to_string(),
        url: resolved.url,
        target: Some(final_path),
    })
}

/// Entry label: drop hosting-host prefix noise down to the row label.
fn entry_label(e: &DownloadEntry) -> String {
    if e.label.trim().is_empty() {
        e.host.clone()
    } else {
        e.label.trim().to_string()
    }
}

/// The full CLI entry point (produces the exit code for the caller).
#[allow(clippy::too_many_arguments)]
pub fn run(
    ctx: &Context,
    game: &str,
    version: &str,
    platform: &str,
    tab: &str,
    _resume: bool,
    _queue: bool,
) -> Result<crate::cli::ExitCode, Error> {
    let sel = Select {
        game,
        version,
        platform,
        tab,
    };
    run_with(
        ctx,
        &sel,
        &mut |url| scraper::fetch(url),
        &mut resolver::resolve,
        &mut dispatch_job,
    )
}

/// Spawn a single job via its adapter; errors stop the batch before any
/// manager receives a URL (so the batch is atomic on pre-checks).
fn dispatch_job(manager: &str, job: &Job) -> Result<(), Error> {
    let adapter = dm::find(manager).ok_or_else(|| {
        Error::DmMissing(format!(
            "active download manager '{manager}' is not installed"
        ))
    })?;
    let (dir, name) = match &job.target {
        Some(p) => {
            let dir = p.parent().map(|d| d.to_path_buf()).unwrap_or_default();
            let name = p
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_default();
            (dir, Some(name))
        }
        None => (PathBuf::new(), None),
    };
    adapter.launch(&job.url, &dir, name.as_deref())
}

/// Everything except the network+spawn seams; wired for tests with fixtures.
pub fn run_with(
    ctx: &Context,
    sel: &Select<'_>,
    fetch: &mut dyn FnMut(&str) -> Result<String, Error>,
    resolve: &mut dyn FnMut(&str) -> Result<resolver::ResolvedUrl, Error>,
    dispatch: &mut dyn FnMut(&str, &Job) -> Result<(), Error>,
) -> Result<crate::cli::ExitCode, Error> {
    if sel.game.trim().is_empty() {
        return Err(Error::Usage("game slug required".to_string()));
    }
    // Canonical game URL (slug may itself be a full URL; normalize).
    let slug = crate::core::game_arg_slug(sel.game);
    let url = format!("https://lewdzone.com/game/{slug}/");
    let html = fetch(&url)?;
    let parsed = scraper::game::parse_game(&html);

    let entries = pick_entries(&parsed, sel.version, sel.platform, sel.tab)?;
    let manager = dm::active_name(ctx)?;
    if dm::find(&manager).is_none() {
        let names = dm::available_names();
        return Err(Error::DmMissing(format!(
            "active download manager '{manager}' is not available on this platform (available: {})",
            names.join(", ")
        )));
    }
    let root = dm::download_root(ctx)?;

    let mut jobs: Vec<Job> = Vec::new();
    for entry in entries {
        jobs.push(job_for(&parsed, entry, &manager, &root, resolve)?);
    }
    for job in &jobs {
        dispatch(&manager, job)?;
    }

    print!("{}", serde_json::to_string_pretty(&jobs).unwrap());
    Ok(crate::cli::ExitCode::Ok)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::*;

    const GAME_FIXTURE: &str = include_str!("../../tests/fixtures/html/lz_game.html");

    fn sample_game() -> Game {
        scraper::game::parse_game(GAME_FIXTURE)
    }

    fn sample_entry(game: &Game) -> &DownloadEntry {
        game.versions
            .first()
            .and_then(|v| v.official.first())
            .expect("fixture game has an official entry")
    }

    fn stub_resolve(url: &str) -> Result<resolver::ResolvedUrl, Error> {
        Ok(resolver::ResolvedUrl {
            url: format!("https://fileknot.io/dl/{}", url),
            host: "fileknot".to_string(),
            platform: Some("pc".to_string()),
            version: None,
            game_id: None,
        })
    }

    #[test]
    fn pick_entries_finds_latest_official_windows() {
        let game = sample_game();
        let entries = pick_entries(&game, "latest", "PC", "official").unwrap();
        assert!(!entries.is_empty());
        assert_eq!(entries[0].host, "fileknot");
    }

    #[test]
    fn pick_entries_rejects_unknown_version() {
        let game = sample_game();
        let err = pick_entries(&game, "99.9", "PC", "official").unwrap_err();
        assert!(matches!(err, Error::Usage(_)));
        assert!(err.to_string().contains("1.0117"));
    }

    #[test]
    fn pick_entries_rejects_missing_platform() {
        let game = sample_game();
        let err = pick_entries(&game, "latest", "Atari", "official").unwrap_err();
        assert!(matches!(err, Error::Usage(_)));
        assert!(err.to_string().contains("atari"));
    }

    #[test]
    fn pick_entries_rejects_unknown_tab() {
        let game = sample_game();
        let err = pick_entries(&game, "latest", "PC", "weird").unwrap_err();
        assert!(matches!(err, Error::Usage(_)));
        assert!(err.to_string().contains("expected 'official'"));
    }

    #[test]
    fn job_for_builds_canonical_target() {
        let game = sample_game();
        let entry = sample_entry(&game);
        let root = std::path::Path::new("D:/Downloads");
        let job = job_for(&game, entry, "fdm", root, &mut stub_resolve).unwrap();
        assert!(job.url.starts_with("https://fileknot.io/"));
        let target = job.target.unwrap();
        assert!(target.starts_with(root));
        assert!(target.extension().is_some());
    }

    #[test]
    fn run_with_offline_uses_fixture_fetch() {
        let ctx = Context::new(
            std::env::temp_dir().join("lz-dl.db"),
            std::env::temp_dir().join("lz-dl-config.json"),
        );
        let mut dispatched: Vec<String> = Vec::new();
        let sel = Select {
            game: "treasure-of-nadia",
            version: "latest",
            platform: "PC",
            tab: "official",
        };
        let code = run_with(
            &ctx,
            &sel,
            &mut |url| {
                assert!(url.ends_with("//lewdzone.com/game/treasure-of-nadia/"));
                Ok(GAME_FIXTURE.to_string())
            },
            &mut stub_resolve,
            &mut |_mgmt, job| {
                dispatched.push(job.url.clone());
                let _ =
                    std::fs::create_dir_all(job.target.as_ref().and_then(|p| p.parent()).unwrap());
                Ok(())
            },
        );
        assert!(code.is_ok());
        assert_eq!(code.unwrap(), crate::cli::ExitCode::Ok);
        assert!(!dispatched.is_empty());
    }
}
