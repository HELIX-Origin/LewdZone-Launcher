//! `search` — search the local catalog (Phase 2).

use crate::core::{not_yet, Context, Error};

pub fn run(ctx: &Context, query: &str) -> Result<crate::cli::ExitCode, Error> {
    let _ = (ctx, query);
    Err(not_yet("search"))
}
