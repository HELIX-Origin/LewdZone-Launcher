//! `launch` — start an installed game (Phase 4).

use crate::core::{not_yet, Context, Error};

pub fn run(ctx: &Context, game: &str) -> Result<crate::cli::ExitCode, Error> {
    let _ = (ctx, game);
    Err(not_yet("launch"))
}
