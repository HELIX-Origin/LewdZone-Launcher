//! `sync` — refresh the catalog from lewdzone.com (Phase 2).

use crate::core::{not_yet, Context, Error};

pub fn run(
    ctx: &Context,
    full: bool,
    platform: Option<&str>,
) -> Result<crate::cli::ExitCode, Error> {
    let _ = (ctx, full, platform);
    Err(not_yet("sync"))
}
