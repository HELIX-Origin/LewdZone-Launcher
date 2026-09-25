//! Forward-only SQL migrations (Rule 06 / ADR-0003). Applied in order by
//! `db::migrate`, tracked in `schema_migrations`. Never rewrite history —
//! later migrations simply add/modify via new numbered entries.

/// `(name, sql)` pairs applied in lexical order.
///
/// 001_initial: base catalog schema from schema-designer's erDiagram and
/// Rule 06's target schema — games, genres, versions, download entries
/// (tokens only, ADR-0003), hosts, jobs, genres join, sync_state.
/// 002_secrets: API keys and other secrets live in SQLite (Rule 10) — the
/// JSON config never holds them, so no plaintext key can sit in a readable
/// file. Presence is surfaced to the UI; values are never echoed.
pub const MIGRATIONS: &[(&str, &str)] = &[
    (
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
    ),
    (
        "002_secrets",
        r#"
CREATE TABLE IF NOT EXISTS secret (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
"#,
    ),
    (
        "003_queue_jobs",
        r#"
CREATE TABLE IF NOT EXISTS queue_job (
    id          INTEGER PRIMARY KEY,
    slug        TEXT    NOT NULL,
    version     TEXT    NOT NULL,
    platform    TEXT    NOT NULL,
    tab         TEXT    NOT NULL,
    source      TEXT,
    status      TEXT    NOT NULL,
    message     TEXT,
    bytes_done  INTEGER NOT NULL DEFAULT 0,
    bytes_total INTEGER NOT NULL DEFAULT 0,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_queue_job_status ON queue_job(status);
"#,
    ),
    (
        "004_content_artwork",
        r#"
CREATE TABLE IF NOT EXISTS game_external (
    post_id     INTEGER NOT NULL REFERENCES game(post_id) ON DELETE CASCADE,
    provider    TEXT    NOT NULL,
    external_id TEXT    NOT NULL,
    data        TEXT    NOT NULL DEFAULT '{}',
    updated_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    PRIMARY KEY (post_id, provider)
);

CREATE TABLE IF NOT EXISTS artwork_cache (
    key         TEXT    NOT NULL,
    kind        TEXT    NOT NULL,
    provider    TEXT    NOT NULL,
    file_path   TEXT    NOT NULL,
    fetched_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    PRIMARY KEY (key, kind)
);
"#,
    ),
    (
        "005_favorite",
        r#"
CREATE TABLE IF NOT EXISTS favorite (
    post_id     INTEGER PRIMARY KEY REFERENCES game(post_id) ON DELETE CASCADE,
    created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE INDEX IF NOT EXISTS idx_favorite_created ON favorite(created_at);
"#,
    ),
    (
        "006_game_playtime",
        r#"
CREATE TABLE IF NOT EXISTS game_stats (
    slug             TEXT PRIMARY KEY,
    playtime_seconds INTEGER NOT NULL DEFAULT 0,
    play_count       INTEGER NOT NULL DEFAULT 0,
    last_played_at   TEXT,
    created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE INDEX IF NOT EXISTS idx_game_stats_last_played ON game_stats(last_played_at);
"#,
    ),
    (
        "007_game_screenshots",
        r#"
ALTER TABLE game ADD COLUMN screenshots TEXT;
"#,
    ),
    (
        "008_queue_job_title",
        r#"
ALTER TABLE queue_job ADD COLUMN title TEXT;
"#,
    ),
];
