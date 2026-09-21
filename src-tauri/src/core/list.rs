//! `list` — list catalog/library/jobs (Phase 2).

use crate::core::{not_yet, Context, Error};

pub fn run(ctx: &Context, library: bool, jobs: bool) -> Result<crate::cli::ExitCode, Error> {
    let _ = (ctx, library, jobs);
    Err(not_yet("list"))
}
