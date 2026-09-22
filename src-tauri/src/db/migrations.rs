//! Forward-only SQL migrations (Rule 06 / ADR-0003). Applied in order by
//! `db::migrate`, tracked in `schema_migrations`. Never rewrite history —
//! later migrations simply add/modify via new numbered entries.

/// `(name, sql)` pairs applied in lexical order.
///
/// 001_initial: base catalog schema from schema-designer's erDiagram and
/// Rule 06's target schema — games, genres, versions, download entries
/// (tokens only, ADR-0003), hosts, jobs, genres join, sync_state.
pub const MIGRATIONS: &[(&str, &str)] = &[(
    "001_initial",
    r#"
CREATE TABLE IF NOT EXISTS game (
    post_id         INTEGER PRIMARY KEY,
    slug            TEXT    NOT NULL UNIQUE,
    title           TEXT    NOT NULL,
    developer       TEXT,
    engine          TEXT,
    size_label      TEXT,
    censorship      TEXT,
    thumbnail_url   TEXT,
    description     TEXT,
    updated_at      TEXT
);

CREATE TABLE IF NOT EXISTS genre (
    slug    TEXT PRIMARY KEY,
    label   TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS game_genre (
    game_id     INTEGER NOT NULL REFERENCES game(post_id) ON DELETE CASCADE,
    genre_id    TEXT    NOT NULL REFERENCES genre(slug)  ON DELETE CASCADE,
    PRIMARY KEY (game_id, genre_id)
);
CREATE INDEX IF NOT EXISTS idx_game_genre_genre ON game_genre(genre_id);

CREATE TABLE IF NOT EXISTS version (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id     INTEGER NOT NULL REFERENCES game(post_id) ON DELETE CASCADE,
    label       TEXT    NOT NULL,
    is_latest   INTEGER NOT NULL DEFAULT 0,
    UNIQUE (game_id, label)
);
CREATE INDEX IF NOT EXISTS idx_version_game ON version(game_id);

CREATE TABLE IF NOT EXISTS host (
    slug          TEXT PRIMARY KEY,
    display_name  TEXT
);

CREATE TABLE IF NOT EXISTS download_entry (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id     INTEGER NOT NULL REFERENCES game(post_id) ON DELETE CASCADE,
    version_id  INTEGER NOT NULL REFERENCES version(id)   ON DELETE CASCADE,
    host_slug   TEXT    NOT NULL REFERENCES host(slug),
    platform    TEXT    NOT NULL DEFAULT '',
    tab         TEXT    NOT NULL DEFAULT '',
    label       TEXT    NOT NULL DEFAULT '',
    variant     TEXT,
    token       TEXT    NOT NULL,
    UNIQUE (game_id, version_id, host_slug, platform, tab, label, variant)
);
CREATE INDEX IF NOT EXISTS idx_download_entry_game ON download_entry(game_id);
CREATE INDEX IF NOT EXISTS idx_download_entry_version ON download_entry(version_id);
CREATE INDEX IF NOT EXISTS idx_download_entry_host ON download_entry(host_slug);

CREATE TABLE IF NOT EXISTS download_job (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_id    INTEGER NOT NULL REFERENCES download_entry(id),
    status      TEXT    NOT NULL DEFAULT 'queued',
    target_dir  TEXT,
    dm_name     TEXT,
    kind        TEXT,
    data_path   TEXT,
    created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE INDEX IF NOT EXISTS idx_download_job_entry ON download_job(entry_id);
CREATE INDEX IF NOT EXISTS idx_download_job_status ON download_job(status);

CREATE TABLE IF NOT EXISTS sync_state (
    key     TEXT PRIMARY KEY,
    value   TEXT NOT NULL
);
"#,
)];
