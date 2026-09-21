"""Initial schema: catalog geometry, downloads, jobs, enrichment (Rule 06 erDiagram
+ ADR-0003 game_external / artwork_cache).

Tokens are stored, never resolved URLs (ADR-0003); artwork_cache carries the
content-provider layer columns (provider + kind, ADR-0004).
"""

from __future__ import annotations

import sqlite3


def upgrade(conn: sqlite3.Connection) -> None:
    conn.executescript(
        """
        CREATE TABLE game (
            post_id       INTEGER PRIMARY KEY,
            slug          TEXT NOT NULL,
            title         TEXT NOT NULL,
            developer     TEXT,
            engine        TEXT,
            size_bytes    INTEGER,
            censorship    TEXT,
            uncensored    INTEGER NOT NULL DEFAULT 0,
            thumbnail_url TEXT
        );
        CREATE UNIQUE INDEX idx_game_slug ON game(slug);

        CREATE TABLE genre (
            slug  TEXT PRIMARY KEY,
            label TEXT NOT NULL
        );

        CREATE TABLE game_genre (
            game_post_id INTEGER NOT NULL REFERENCES game(post_id) ON DELETE CASCADE,
            genre_slug   TEXT NOT NULL REFERENCES genre(slug) ON DELETE CASCADE,
            PRIMARY KEY (game_post_id, genre_slug)
        );

        CREATE TABLE version (
            id       INTEGER PRIMARY KEY,
            game_id  INTEGER NOT NULL REFERENCES game(post_id) ON DELETE CASCADE,
            label    TEXT NOT NULL,
            is_latest INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE host (
            slug         TEXT PRIMARY KEY,
            display_name TEXT
        );

        CREATE TABLE download_entry (
            id          INTEGER PRIMARY KEY,
            game_id     INTEGER NOT NULL REFERENCES game(post_id) ON DELETE CASCADE,
            version_id  INTEGER NOT NULL REFERENCES version(id) ON DELETE CASCADE,
            host_slug   TEXT NOT NULL REFERENCES host(slug),
            platform    TEXT NOT NULL,
            tab         TEXT NOT NULL DEFAULT 'official',
            label       TEXT,
            token       TEXT NOT NULL
        );

        CREATE TABLE download_job (
            id         INTEGER PRIMARY KEY,
            entry_id   INTEGER NOT NULL REFERENCES download_entry(id) ON DELETE CASCADE,
            status     TEXT NOT NULL DEFAULT 'queued',
            target_dir TEXT NOT NULL,
            dm_name    TEXT,
            kind       TEXT NOT NULL DEFAULT 'http',
            data_path  TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE game_external (
            post_id     INTEGER NOT NULL REFERENCES game(post_id) ON DELETE CASCADE,
            provider    TEXT NOT NULL,
            external_id TEXT NOT NULL,
            title       TEXT,
            fetched_at  TEXT NOT NULL DEFAULT (datetime('now')),
            PRIMARY KEY (post_id, provider)
        );

        CREATE TABLE artwork_cache (
            id        INTEGER PRIMARY KEY,
            game_id   INTEGER NOT NULL REFERENCES game(post_id) ON DELETE CASCADE,
            provider  TEXT,
            kind      TEXT NOT NULL,
            path      TEXT NOT NULL,
            UNIQUE (game_id, provider, kind)
        );
        """
    )