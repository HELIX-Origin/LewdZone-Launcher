//! `search` — browse or search the lewdzone.com archive.
//! Shares `core::catalog` with the Store view (Rule 03 GUI/CLI parity).

use crate::cli::ExitCode;
use crate::core::{catalog, Context, Error};

pub fn run(ctx: &Context, query: &str) -> Result<ExitCode, Error> {
    if !query.trim().is_empty() {
        return Err(Error::Usage(
            "live free-text search endpoint not wired yet — browse the archive with `sync`/`list` (Rule 12: usage error)"
                .to_string(),
        ));
    }
    let parsed = catalog::archive_page(ctx, 1, Some("Popularity"), None)?;
    println!("{}", serde_json::to_string_pretty(&parsed)?);
    Ok(ExitCode::Ok)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Context;

    fn ctx() -> Context {
        Context::new(
            std::path::PathBuf::from("lewdzone-test.db"),
            std::path::PathBuf::from("lewdzone-test.json"),
        )
    }

    #[test]
    fn free_text_query_is_unimplemented_with_usage_error() {
        let err = run(&ctx(), "treasure of nadia").unwrap_err();
        assert!(matches!(err, Error::Usage(_)));
    }

    #[test]
    fn archive_url_builder_is_pure() {
        assert_eq!(
            catalog::archive_url(1, Some("Popularity"), None),
            "https://lewdzone.com/games/?sort=Popularity"
        );
        assert_eq!(
            catalog::archive_url(2, None, Some("PC")),
            "https://lewdzone.com/games/page/2/?platform=PC"
        );
    }
}
