//! `info` — show one game's detail (Phase 2).

use crate::core::{not_yet, Context, Error};

pub fn run(ctx: &Context, game: &str, versions: bool) -> Result<crate::cli::ExitCode, Error> {
    let _ = (ctx, game, versions);
    Err(not_yet("info"))
}
