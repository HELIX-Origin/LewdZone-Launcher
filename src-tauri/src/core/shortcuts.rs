//! Native per-OS shortcuts for installed games (discontinued, ADR-0006).
//!
//! Desktop shortcut support was abandoned because official site pages do not
//! provide fitting square icon images for many games to serve as desktop icons.

use std::path::PathBuf;

use crate::cli::ExitCode;
use crate::core::{Context, Error};

/// Create a Desktop shortcut for an installed game (discontinued).
pub fn create(_ctx: &Context, _slug: &str) -> Result<PathBuf, Error> {
    Err(Error::Usage(
        "desktop shortcut support has been discontinued (ADR-0006)".to_string(),
    ))
}

/// CLI entry point for `lewdzone shortcuts [--game <slug>]`.
pub fn run(_ctx: &Context, _game: Option<&str>, _skip_artwork: bool) -> Result<ExitCode, Error> {
    eprintln!("desktop shortcut support has been discontinued (ADR-0006)");
    Ok(ExitCode::Usage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcut_creation_is_discontinued() {
        let db = PathBuf::from(":memory:");
        let config = PathBuf::from("config.json");
        let ctx = Context::new(db, config);
        let res = create(&ctx, "wild-life");
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("discontinued"));
    }
}
