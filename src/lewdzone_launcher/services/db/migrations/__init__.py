"""Forward-only SQLite migrations (Rule 06). Each module is ``NNN_desc.py``
with an ``upgrade(conn)`` function; applied in lexical order via
``services.db.core.migrate`` and recorded in ``schema_migrations``.
"""