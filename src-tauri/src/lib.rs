//! The Tauri app + native CLI share this crate (Rule 03: one core, two entry
//! points). Webview handlers are `#[tauri::command]`s over `core`; `main.rs`
//! routes argv to `cli_main` or the windowed `run`.

pub mod cli;
pub mod core;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// CLI entry point called from `main.rs` when argv has subcommands.
pub fn cli_main() -> std::process::ExitCode {
    use clap::Parser;

    let cli = cli::Cli::parse();
    let code = cli::run(cli);
    std::process::ExitCode::from(code.as_i32() as u8)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
