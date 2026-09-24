// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args
        .iter()
        .any(|a| a == "--installer" || a == "--setup" || a == "--uninstall" || a == "--maintenance")
    {
        lewdzone_lib::run_installer();
        return ExitCode::SUCCESS;
    }
    // Native CLI: any other argv beyond the binary name dispatches to the CLI
    if args.len() > 1 {
        return lewdzone_lib::cli_main();
    }
    lewdzone_lib::run();
    ExitCode::SUCCESS
}
