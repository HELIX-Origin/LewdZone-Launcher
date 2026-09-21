//! `shortcuts` — native .lnk/.desktop/.app + SteamGridDB artwork (Phase 4).

use crate::core::{not_yet, Context, Error};

pub fn run(
    ctx: &Context,
    game: Option<&str>,
    skip_artwork: bool,
) -> Result<crate::cli::ExitCode, Error> {
    let _ = (ctx, game, skip_artwork);
    Err(not_yet("shortcuts"))
}
