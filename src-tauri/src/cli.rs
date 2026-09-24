//! Native Rust CLI — second entry point into the same binary as the Tauri app.
//!
//! Running the binary with a subcommand (e.g. `lewdzone sync --json`)
//! dispatches here and exits with a stable exit code; running it bare launches
//! the windowed app (see `main.rs`). All behavior lives in the shared core
//! (`crate::core`), never in this parser.

use clap::{Parser, Subcommand};

use crate::core;

/// Exit codes (Rule 12). Stable contract for scripts and the GUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    /// Success.
    Ok = 0,
    /// Runtime error.
    Runtime = 1,
    /// Usage error (bad flags/args).
    Usage = 2,
    /// Network / site error.
    Network = 3,
    /// Interrupted (user cancelled).
    Interrupted = 5,
}

impl ExitCode {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

/// LewdZone Launcher — cross-platform desktop game launcher.
///
/// Bundles a Tauri 2 desktop app and this native CLI behind one Rust core.
/// `--json` emits one machine-readable document on stdout; progress and
/// diagnostics go to stderr.
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    /// Override the SQLite database path.
    #[arg(long, global = true)]
    db: Option<String>,

    /// Override the config file path.
    #[arg(long, global = true)]
    config: Option<String>,

    /// Emit machine-readable JSON on stdout.
    #[arg(long, global = true)]
    json: bool,

    /// Show diagnostic detail on stderr.
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Disable colorized output.
    #[arg(long, global = true)]
    no_color: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Refresh the catalog from lewdzone.com.
    Sync(SyncArgs),
    /// Search the catalog.
    Search(SearchArgs),
    /// Show details for one game.
    Info(InfoArgs),
    /// Queue or resume a download for a game release.
    Download(DownloadArgs),
    /// List catalog or job state.
    List(ListArgs),
    /// Read or write settings.
    Settings(SettingsArgs),
    /// Manage native shortcuts.
    Shortcuts(ShortcutsArgs),
    /// Launch an installed game.
    Launch(LaunchArgs),
    /// Manage favorite games.
    Favorites(FavoritesArgs),
}

#[derive(clap::Args, Debug, Default)]
struct SyncArgs {
    /// Full resync (all pages) instead of incremental.
    #[arg(long)]
    full: bool,
    /// Filter platforms, e.g. `--platform PC`.
    #[arg(long)]
    platform: Option<String>,
}

#[derive(clap::Args, Debug, Default)]
struct SearchArgs {
    /// Search query.
    query: String,
}

#[derive(clap::Args, Debug, Default)]
struct InfoArgs {
    /// The game: slug, post id, or URL.
    game: Option<String>,
    /// The game (alternative to the positional form).
    #[arg(long = "game", conflicts_with = "game")]
    game_flag: Option<String>,
    /// List versions instead of the summary.
    #[arg(long)]
    versions: bool,
}

#[derive(clap::Args, Debug, Default)]
struct DownloadArgs {
    /// The game: slug, post id, or URL.
    game: Option<String>,
    /// The game (alternative to the positional form).
    #[arg(long = "game", conflicts_with = "game")]
    game_flag: Option<String>,
    /// Version label (e.g. `1.0`), or `latest`.
    #[arg(long, default_value = "latest")]
    version: String,
    /// Platform (PC, macOS, Linux, Android).
    #[arg(long, default_value = "PC")]
    platform: String,
    /// Download tab: official or community.
    #[arg(long, default_value = "official")]
    tab: String,
    /// Restrict to one source host (e.g. `mega`, `google`).
    #[arg(long)]
    source: Option<String>,
    /// Resume an existing job instead of starting fresh.
    #[arg(long)]
    resume: bool,
    /// Add to the queue without starting.
    #[arg(long)]
    queue: bool,
}

#[derive(clap::Args, Debug, Default)]
struct ListArgs {
    /// List installed/library games instead of the catalog.
    #[arg(long)]
    library: bool,
    /// Show the job queue.
    #[arg(long)]
    jobs: bool,
}

#[derive(clap::Args, Debug, Default)]
struct SettingsArgs {
    #[command(subcommand)]
    command: Option<SettingsCmd>,
}

#[derive(Subcommand, Debug)]
enum SettingsCmd {
    /// Read one or all settings.
    Get(GetArgs),
    /// Set a setting value.
    Set(SetArgs),
}

#[derive(clap::Args, Debug, Default)]
struct GetArgs {
    /// Setting key (omit for all).
    key: Option<String>,
}

#[derive(clap::Args, Debug, Default)]
struct SetArgs {
    /// Setting key, e.g. `download-root`.
    key: String,
    /// Value.
    value: String,
    /// Store the value as a secret (API key) in the SQLite DB — never echoed.
    #[arg(long)]
    secret: bool,
}

#[derive(clap::Args, Debug, Default)]
struct ShortcutsArgs {
    /// The installed game slug.
    slug: Option<String>,
    /// Rebuild shortcuts for one game.
    #[arg(long)]
    game: Option<String>,
    /// Skip fetching artwork from SteamGridDB.
    #[arg(long)]
    skip_artwork: bool,
}

#[derive(clap::Args, Debug, Default)]
struct LaunchArgs {
    /// The installed game: slug.
    game: String,
}

#[derive(clap::Args, Debug, Default)]
struct FavoritesArgs {
    #[command(subcommand)]
    command: FavoritesCmd,
}

#[derive(Subcommand, Debug, Default)]
enum FavoritesCmd {
    /// List favorited games.
    #[default]
    List,
    /// Add a game to favorites.
    Add { slug: String },
    /// Remove a game from favorites.
    Remove { slug: String },
}

/// Dispatch a parsed CLI invocation, printing results, returning the exit code.
pub fn run(cli: Cli) -> ExitCode {
    let result = dispatch(cli);
    match result {
        Ok(code) => code,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(&err)
        }
    }
}

fn dispatch(cli: Cli) -> Result<ExitCode, crate::core::Error> {
    use core::paths;
    let db = cli
        .db
        .map(std::path::PathBuf::from)
        .or_else(paths::default_db_path);
    let config = cli
        .config
        .map(std::path::PathBuf::from)
        .or_else(paths::default_config_path);
    let ctx = core::Context::new(db.unwrap(), config.unwrap());

    match cli.command {
        Command::Sync(args) => core::sync::run(&ctx, args.full, args.platform.as_deref()),
        Command::Search(args) => core::search::run(&ctx, &args.query),
        Command::Info(args) => {
            let game = args.game.or(args.game_flag).ok_or_else(|| {
                core::Error::Usage("a game (slug, post id, or URL) is required".into())
            })?;
            core::info::run(&ctx, &game, args.versions)
        }
        Command::Download(args) => {
            let game = args.game.or(args.game_flag).ok_or_else(|| {
                core::Error::Usage("a game (slug, post id, or URL) is required".into())
            })?;
            core::download::run(
                &ctx,
                &game,
                &args.version,
                &args.platform,
                &args.tab,
                args.source.as_deref(),
                args.resume,
                args.queue,
            )
        }
        Command::List(args) => core::list::run(&ctx, args.library, args.jobs),
        Command::Settings(args) => match args.command {
            Some(SettingsCmd::Get(g)) => core::settings::get(&ctx, g.key.as_deref()),
            Some(SettingsCmd::Set(s)) => core::settings::set(&ctx, &s.key, &s.value, s.secret),
            None => core::settings::get(&ctx, None),
        },
        Command::Shortcuts(args) => {
            let slug = args.slug.as_deref().or(args.game.as_deref());
            core::shortcuts::run(&ctx, slug, args.skip_artwork)
        }
        Command::Launch(args) => core::launch::run(&ctx, &args.game),
        Command::Favorites(args) => match args.command {
            FavoritesCmd::List => {
                let games = core::favorites::list(&ctx)?;
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&games)?);
                } else {
                    if games.is_empty() {
                        println!("No favorites yet.");
                    } else {
                        println!("Favorites:");
                        for g in games {
                            println!("  {}", g.title);
                        }
                    }
                }
                Ok(ExitCode::Ok)
            }
            FavoritesCmd::Add { slug } => {
                core::favorites::add(&ctx, &slug)?;
                if !cli.json {
                    println!("Added '{}' to favorites.", slug);
                }
                Ok(ExitCode::Ok)
            }
            FavoritesCmd::Remove { slug } => {
                core::favorites::remove(&ctx, &slug)?;
                if !cli.json {
                    println!("Removed '{}' from favorites.", slug);
                }
                Ok(ExitCode::Ok)
            }
        },
    }
}

impl From<&crate::core::Error> for ExitCode {
    fn from(err: &crate::core::Error) -> Self {
        match err {
            crate::core::Error::Usage(_) => ExitCode::Usage,
            crate::core::Error::Network(_) => ExitCode::Network,
            crate::core::Error::Interrupted => ExitCode::Interrupted,
            crate::core::Error::Runtime(_) => ExitCode::Runtime,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sync_with_json_flag() {
        let cli = Cli::try_parse_from(["lewdzone", "sync", "--json", "--full"]).unwrap();
        assert!(cli.json);
        match cli.command {
            Command::Sync(a) => assert!(a.full),
            other => panic!("expected sync, got {other:?}"),
        }
    }

    #[test]
    fn parses_download_with_flags() {
        let cli = Cli::try_parse_from([
            "lewdzone",
            "download",
            "treasure-of-nadia",
            "--version",
            "1.0",
            "--platform",
            "PC",
            "--tab",
            "official",
        ])
        .unwrap();
        match cli.command {
            Command::Download(a) => {
                assert_eq!(a.game.as_deref(), Some("treasure-of-nadia"));
                assert_eq!(a.version, "1.0");
                assert_eq!(a.platform, "PC");
                assert_eq!(a.tab, "official");
            }
            other => panic!("expected download, got {other:?}"),
        }
    }

    #[test]
    fn parses_game_via_flag_form() {
        let cli =
            Cli::try_parse_from(["lewdzone", "info", "--game", "treasure-of-nadia", "--json"])
                .unwrap();
        match cli.command {
            Command::Info(a) => {
                assert_eq!(a.game_flag.as_deref(), Some("treasure-of-nadia"));
                assert!(a.game.is_none());
            }
            other => panic!("expected info, got {other:?}"),
        }
    }

    #[test]
    fn rejects_game_in_both_forms() {
        let cli = Cli::try_parse_from(["lewdzone", "info", "treasure-of-nadia", "--game", "other"]);
        assert!(cli.is_err());
    }

    #[test]
    fn rejects_unknown_subcommand() {
        assert!(Cli::try_parse_from(["lewdzone", "frobnicate"]).is_err());
    }

    #[test]
    fn parses_favorites_subcommands() {
        let list = Cli::try_parse_from(["lewdzone", "favorites", "list", "--json"]).unwrap();
        assert!(list.json);
        match list.command {
            Command::Favorites(a) => matches!(a.command, FavoritesCmd::List),
            _ => false,
        };

        let add = Cli::try_parse_from(["lewdzone", "favorites", "add", "wild-life"]).unwrap();
        match add.command {
            Command::Favorites(a) => matches!(a.command, FavoritesCmd::Add { .. }),
            _ => false,
        };

        let remove = Cli::try_parse_from(["lewdzone", "favorites", "remove", "wild-life"]).unwrap();
        match remove.command {
            Command::Favorites(a) => matches!(a.command, FavoritesCmd::Remove { .. }),
            _ => false,
        };
    }

    #[test]
    fn exit_codes_are_stable_contract() {
        assert_eq!(ExitCode::Ok.as_i32(), 0);
        assert_eq!(ExitCode::Runtime.as_i32(), 1);
        assert_eq!(ExitCode::Usage.as_i32(), 2);
        assert_eq!(ExitCode::Network.as_i32(), 3);
        assert_eq!(ExitCode::Interrupted.as_i32(), 5);
    }
}
