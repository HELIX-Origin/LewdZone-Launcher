# 🐛 BUGS (open issues)

Only open bugs belong here. Each entry links to its GitHub issue (once filed)
and is closed by editing this file, not by deleting history. When a bug is fixed,
move it to the commit that resolved it (`git log`).

## 2026-09-23 — `core::favorites` roundtrip test fails on Windows (#<issue>)

- **Steps to reproduce:** Run `cargo test --lib core::favorites::tests::favorites_roundtrip_by_slug` on Windows.
- **Expected:** Test passes.
- **Actual:** `SqliteFailure(CannotOpen, extended_code 14)`: unable to open database file at `C:\Users\...\AppData\Local\Temp\lz-fav-roundtrip-<pid>\lewdzone.db`.
- **Root cause:** The test helper in `src-tauri/src/core/favorites.rs` removes the temp dir but never creates it before opening the SQLite database.
- **Environment:** Windows 11, Rust 1.85, commit `a9265fb`.
- [ ] Fix in PR (`Closes #<issue>`)

## 📝 Filing a bug

## 📝 Filing a bug

Bug title on GitHub: `🐛 <problem summary>`. Body must include:

- Steps to reproduce (reproduce-first)
- Expected vs actual behavior
- Environment (OS, app/CLI version, DM in use)
- ≥1 verifiable diagram or log when applicable ([Rule 04](.agents/rules/rule-04-remote-issue-protocol.md))

Entry format once filed:

```
## 2026-09-21 — <short title>  (#<issue>)
- [ ] Reproduced
- [ ] Root cause identified
- [ ] Fix in PR (`Closes #<issue>`)
```