//! `sync` — refresh the catalog from lewdzone.com (owner: database /
//! sync-orchestrator, Rule 06, ADR-0003, Rule 05).
//!
//! Pipeline per sync-orchestrator:
//!   1. Walk archive pages with the polite fetcher.
//!   2. Change-detect each card against the stored `updated_at`
//!      (catalog/sync Change-Detection Basis). Unchanged cards are skipped.
//!   3. Fetch the game page for new/changed cards and parse it.
//!   4. Upsert per page inside one transaction (write-all-or-nothing per page).
//!   5. Record the last successful page in `sync_state` for resumability.
//!   6. Prune games that vanished — ONLY after a clean full sync; a failed or
//!      partial sync never prunes.
//!
//! The network boundary (`fetch_archive`, `fetch_game`) is injected so the
//! whole pipeline is testable offline with committed fixtures (Rule 11).

use std::collections::HashSet;

use rusqlite::Connection;

use crate::cli::ExitCode;
use crate::core::catalog;
use crate::core::models::{ArchiveMeta, Game, GameCard};
use crate::core::{Context, Error};
use crate::scraper::archive::ArchivePage;
use crate::{db, scraper};

/// What one `sync` run did (for CLI output / GUI progress).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SyncReport {
    pub pages_synced: u32,
    pub games_synced: u32,
    pub games_pruned: u32,
    pub games_unchanged: u32,
    pub games_skipped: u32,
    pub resume_page: u32,
}

impl SyncReport {
    pub fn fmt_summary(&self) -> String {
        format!(
            "synced {} page(s), imported {} game(s) ({} unchanged, {} skipped), pruned {}",
            self.pages_synced,
            self.games_synced,
            self.games_unchanged,
            self.games_skipped,
            self.games_pruned
        )
    }
}

/// Production entry point: real network + the user catalog DB (Rule 13).
pub fn run(ctx: &Context, full: bool, platform: Option<&str>) -> Result<ExitCode, Error> {
    let conn = db::open(&ctx.db_path)?;
    db::migrate(&conn)?;

    let report = run_with(
        &conn,
        full,
        platform,
        &mut |page, platform| catalog::archive_page(ctx, page, None, platform),
        &mut |card| {
            let url = format!("https://lewdzone.com/game/{}/", card.slug);
            let html = scraper::fetch(&url)?;
            Ok(scraper::game::parse_game(&html))
        },
    )?;

    println!("{}", report.fmt_summary());
    Ok(ExitCode::Ok)
}

/// Testable sync engine. All network access goes through the two closures:
/// `fetch_archive(page, platform) -> ArchivePage`, then per changed/new card
/// `fetch_game(&GameCard) -> Game`. Commits once per page; records the last
/// successful page; prunes only on a clean full sync.
pub fn run_with<F, G>(
    conn: &Connection,
    full: bool,
    platform: Option<&str>,
    fetch_archive: &mut F,
    fetch_game: &mut G,
) -> Result<SyncReport, Error>
where
    F: FnMut(u32, Option<&str>) -> Result<ArchivePage, Error>,
    G: FnMut(&GameCard) -> Result<Game, Error>,
{
    // Resume: a non-full sync continues after the last successful page.
    let start = if full {
        1
    } else {
        db::repo::sync_state_get(conn, "last_page")?
            .and_then(|p| p.parse::<u32>().ok())
            .map(|p| p + 1)
            .unwrap_or(1)
    };

    let mut report = SyncReport {
        resume_page: start,
        ..Default::default()
    };
    let mut seen: HashSet<String> = HashSet::new();
    let mut page = start;

    loop {
        let archive = fetch_archive(page, platform)?;
        let cards = archive.games;
        if cards.is_empty() {
            break;
        }
        for card in &cards {
            seen.insert(card.slug.clone());
        }

        // Fetch only new/changed games up front, so a network failure mid-page
        // aborts before any write for that page (write-all-or-nothing).
        let mut pending: Vec<(Game, Option<String>, Option<String>)> = Vec::new();
        for card in &cards {
            if !is_changed(conn, card)? {
                report.games_unchanged += 1;
                continue;
            }
            let game = fetch_game(card)?;
            pending.push((game, card.updated_at.clone(), card.thumb_url.clone()));
        }

        if !pending.is_empty() {
            let tx = conn.unchecked_transaction()?;
            for (game, updated_at, thumb_url) in &pending {
                if game.post_id.is_some() {
                    db::repo::upsert_game(&tx, game, updated_at.as_deref(), thumb_url.as_deref())?;
                    report.games_synced += 1;
                } else {
                    report.games_skipped += 1;
                }
            }
            tx.commit()?;
        }
        report.pages_synced += 1;

        let state_tx = conn.unchecked_transaction()?;
        db::repo::sync_state_set(&state_tx, "last_page", &page.to_string())?;
        state_tx.commit()?;

        // Termination: after the site's known last page, or when a page comes
        // back empty.
        let pages_total = archive.meta.total_pages.unwrap_or(page);
        if page >= pages_total {
            break;
        }
        page += 1;
    }

    if full && !seen.is_empty() {
        let tx = conn.unchecked_transaction()?;
        report.games_pruned = db::repo::prune_slugs(&tx, &seen)?;
        tx.commit()?;
    }
    Ok(report)
}

/// Change-detection basis (sync-orchestrator: Catalog/Sync subsection):
/// a card needs a re-fetch when there is no stored game for its slug or the
/// stored `updated_at` differs from the card's.
fn is_changed(conn: &Connection, card: &GameCard) -> Result<bool, Error> {
    let stored = db::repo::game_updated_at(conn, &card.slug)?;
    match stored {
        None => Ok(true),                                              // brand new
        Some(s) => Ok(card.updated_at.as_deref() != Some(s.as_str())), // changed?
    }
}

/// Helpers used by tests and the search page to shape empty metadata.
pub fn empty_archive_meta(page: u32) -> ArchiveMeta {
    catalog::empty_meta(page)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scraper::game::parse_game;

    const ARCHIVE_FIXTURE: &str = include_str!("../../tests/fixtures/html/lz_archive.html");
    const GAME_FIXTURE: &str = include_str!("../../tests/fixtures/html/lz_game.html");

    fn mem_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        db::migrate(&conn).unwrap();
        conn
    }

    /// First `n` cards from the committed archive fixture.
    fn fixture_cards(n: usize) -> Vec<GameCard> {
        scraper::archive::parse_archive(ARCHIVE_FIXTURE)
            .games
            .into_iter()
            .take(n)
            .collect()
    }

    /// A game shaped like the fixture but re-identified to the requested card,
    /// simulating what the upstream game page would return for that card.
    fn fixture_game_for(card: &GameCard) -> Game {
        let mut g = parse_game(GAME_FIXTURE);
        g.slug = card.slug.clone();
        // Archive cards carry no post_id; the game page always does. Derive a
        // deterministic, distinct id per card so the DB sees them separately.
        g.post_id = Some(card.slug.bytes().map(i64::from).sum());
        g.title = card.title.clone();
        g
    }

    fn archive_fetcher(
        cards: Vec<GameCard>,
        total: Option<u32>,
    ) -> impl FnMut(u32, Option<&str>) -> Result<ArchivePage, Error> {
        move |page, _platform| {
            Ok(ArchivePage {
                games: if page == 1 { cards.clone() } else { vec![] },
                meta: ArchiveMeta {
                    page,
                    total_pages: total,
                    applied: Default::default(),
                },
            })
        }
    }

    #[test]
    fn sync_imports_fixture_catalog_offline() {
        let conn = mem_db();
        let cards = fixture_cards(3);
        let gamecard = cards[0].clone();
        let mut cont_archive = archive_fetcher(cards, Some(1));
        let mut cont_game = |card: &GameCard| Ok(fixture_game_for(card));

        let report = run_with(&conn, true, None, &mut cont_archive, &mut cont_game).unwrap();

        assert_eq!(report.pages_synced, 1);
        assert_eq!(report.games_synced, 3);
        assert_eq!(report.games_pruned, 0);
        assert!(db::repo::game_by_slug(&conn, &gamecard.slug)
            .unwrap()
            .is_some());
        assert_eq!(
            db::repo::sync_state_get(&conn, "last_page")
                .unwrap()
                .as_deref(),
            Some("1")
        );
    }

    #[test]
    fn sync_is_idempotent_and_skips_unchanged() {
        let conn = mem_db();
        let cards = fixture_cards(2);
        let mut cont_archive = archive_fetcher(cards, Some(1));
        let mut cont_game = |card: &GameCard| Ok(fixture_game_for(card));

        let first = run_with(&conn, true, None, &mut cont_archive, &mut cont_game).unwrap();
        assert_eq!(first.games_synced, 2);

        // Second identical run: the first archive page was already consumed by
        // cont_archive, so rebuild a fresh fetcher with the same cards.
        let cards2 = fixture_cards(2);
        let mut cont_archive2 = archive_fetcher(cards2, Some(1));
        let second = run_with(&conn, true, None, &mut cont_archive2, &mut cont_game).unwrap();

        assert_eq!(second.games_synced, 0);
        assert_eq!(second.games_unchanged, 2);
        assert_eq!(db::repo::game_count(&conn).unwrap(), 2);
    }

    #[test]
    fn resume_continues_after_last_successful_page() {
        // A full sync of one page commits a game and records last_page=1. A
        // subsequent non-full sync resumes at page 2 (empty here) and is a
        // no-op — page 1 is never re-walked.
        let conn = mem_db();
        let archive_calls = std::cell::Cell::new(0usize);
        let mut cont_archive = |page: u32, _platform: Option<&str>| {
            archive_calls.set(archive_calls.get() + 1);
            Ok(ArchivePage {
                games: if page == 1 { fixture_cards(1) } else { vec![] },
                meta: ArchiveMeta {
                    page,
                    total_pages: None,
                    applied: Default::default(),
                },
            })
        };
        let mut cont_game = |card: &GameCard| Ok(fixture_game_for(card));

        let full = run_with(&conn, true, None, &mut cont_archive, &mut cont_game).unwrap();
        assert_eq!(full.pages_synced, 1);
        assert_eq!(db::repo::game_count(&conn).unwrap(), 1);

        // Non-full sync: resume from last_page=1 → start page 2 → empty → no-op.
        let resume = run_with(&conn, false, None, &mut cont_archive, &mut cont_game).unwrap();
        assert_eq!(resume.resume_page, 2);
        assert_eq!(resume.games_synced, 0);
        assert_eq!(db::repo::game_count(&conn).unwrap(), 1);
        assert_eq!(archive_calls.get(), 2); // 1 full + 1 resume
    }

    #[test]
    fn prune_removes_games_missing_from_next_full_sync() {
        let conn = mem_db();
        // Seed two games.
        let two = fixture_cards(2);
        {
            let mut a1 = archive_fetcher(two, Some(1));
            let mut g1 = |card: &GameCard| Ok(fixture_game_for(card));
            run_with(&conn, true, None, &mut a1, &mut g1).unwrap();
        }
        assert_eq!(db::repo::game_count(&conn).unwrap(), 2);

        // Next full sync only knows one of them → the other gets pruned.
        let one = fixture_cards(1);
        {
            let mut a2 = archive_fetcher(one, Some(1));
            let mut g2 = |card: &GameCard| Ok(fixture_game_for(card));
            let report = run_with(&conn, true, None, &mut a2, &mut g2).unwrap();
            assert_eq!(report.games_pruned, 1);
        }
        assert_eq!(db::repo::game_count(&conn).unwrap(), 1);
    }
}
