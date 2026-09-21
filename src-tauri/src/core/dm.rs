//! `dm` — download-manager detection/selection (Phase 2 + ADR-0002).

use crate::core::{not_yet, Context, Error};

pub fn run(ctx: &Context, active: Option<&str>) -> Result<crate::cli::ExitCode, Error> {
    let _ = (ctx, active);
    Err(not_yet("dm"))
}
