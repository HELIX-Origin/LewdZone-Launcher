// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::ExitCode;

fn main() -> ExitCode {
    // Native CLI: any argv beyond the binary name dispatches to the CLI
    // (Rule 03 two-entry-points). Run bare → launch the windowed app.
    if std::env::args().count() > 1 {
        return lewdzone_launcher_lib::cli_main();
    }
    lewdzone_launcher_lib::run();
    ExitCode::SUCCESS
}
