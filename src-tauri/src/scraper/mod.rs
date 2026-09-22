//! Scraper family — turns lewdzone.com HTML into typed canonical models
//! (owner: `.agents/agents/scraper/scraper.md`). Every parser here is pure
//! `(html) -> model`; the only network entry point is the polite `fetch`
//! helper, which lives in its own module per the scraper spec.

pub mod archive;
pub mod fetch;
pub mod game;
pub use fetch::fetch;
