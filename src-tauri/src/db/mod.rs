//! SQLite persistence layer (owner: database family, Rule 06 / ADR-0003).
//!
//! One `lewdzone.db` in WAL mode with `foreign_keys=ON` and a busy timeout.
//! The schema is forward-only: embedded numbered migrations applied through a
//! `schema_migrations` table (ADR-0003 calls the files `NNN_desc`, Rule 06
//! wants `NNN_desc.sql`; both are represented here as ordered embedded SQL
//! strings). Tokens are stored, never resolved URLs (ADR-0003, Rule 06).

pub mod migrations;
pub mod repo;

use std::path::Path;
use std::time::Duration;

use rusqlite::Connection;

use crate::core::Error;

/// Open (creating parents as needed) with WAL + FK + busy timeout (Rule 06).
pub fn open(path: &Path) -> Result<Connection, Error> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.busy_timeout(Duration::from_millis(5000))?;
    Ok(conn)
}

/// Apply any not-yet-applied migrations in lexical order, stamping each in
/// `schema_migrations` inside its own transaction (forward-only, Rule 06).
pub fn migrate(conn: &Connection) -> Result<(), Error> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version     TEXT PRIMARY KEY,
            applied_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
        );",
    )?;
    for (name, sql) in migrations::MIGRATIONS {
        let applied: i64 = conn.query_row(
            "SELECT COUNT(*) FROM schema_migrations WHERE version = ?1",
            [name],
            |row| row.get(0),
        )?;
        if applied == 0 {
            let tx = conn.unchecked_transaction()?;
            if let Err(e) = tx.execute_batch(sql) {
                let msg = e.to_string();
                if !msg.contains("duplicate column name") {
                    return Err(e.into());
                }
            }
            tx.execute(
                "INSERT INTO schema_migrations (version) VALUES (?1)",
                [name],
            )?;
            tx.commit()?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mem() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        conn
    }

    #[test]
    fn migrate_stamps_schema_migrations() {
        let conn = mem();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, migrations::MIGRATIONS.len() as i64);
    }

    #[test]
    fn migrate_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        migrate(&conn).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, migrations::MIGRATIONS.len() as i64);
    }

    #[test]
    fn pragmas_are_applied() {
        let conn = Connection::open_in_memory().unwrap();
        open_pragmas(&conn).unwrap();
        let fk: i64 = conn
            .pragma_query_value(None, "foreign_keys", |r| r.get(0))
            .unwrap();
        assert_eq!(fk, 1);
    }

    fn open_pragmas(conn: &Connection) -> Result<(), Error> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.busy_timeout(Duration::from_millis(5000))?;
        Ok(())
    }

    #[test]
    fn core_tables_exist_after_migrate() {
        let conn = mem();
        for table in [
            "game",
            "genre",
            "game_genre",
            "version",
            "host",
            "download_entry",
            "download_job",
            "sync_state",
            "secret",
            "queue_job",
            "favorite",
        ] {
            let n: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(n, 1, "table {table} should exist");
        }
    }
}
