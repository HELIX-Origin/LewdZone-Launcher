//! `download` — resolve a token and hand the URL to a download manager
//! (Phase 2 + ADR-0002).

use crate::core::{not_yet, Context, Error};

pub fn run(
    ctx: &Context,
    game: &str,
    version: &str,
    platform: &str,
    tab: &str,
    resume: bool,
    queue: bool,
) -> Result<crate::cli::ExitCode, Error> {
    let _ = (ctx, game, version, platform, tab, resume, queue);
    Err(not_yet("download"))
}
