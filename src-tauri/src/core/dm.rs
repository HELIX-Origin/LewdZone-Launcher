//! `dm` — download-manager detection/selection (Phase 2 + ADR-0002).
//!
//! `lewdzone dm [active]` lists what's installed or flips the active manager.
//! The adapter registry lives in `crate::dm`; this module is the entry point
//! shared by the CLI and (soon) the download pipeline.

use crate::cli::ExitCode;
use crate::core::{Context, Error};
use crate::dm;

pub fn run(ctx: &Context, active: Option<&str>) -> Result<ExitCode, Error> {
    match active {
        Some(name) => activate(ctx, name),
        None => list(ctx),
    }
}

/// `dm <name>` — persist the `dm` setting and report success. Unknown names
/// are a usage error before any settings write.
fn activate(ctx: &Context, name: &str) -> Result<ExitCode, Error> {
    if !dm::available_names().contains(&name) {
        return Err(Error::Usage(format!(
            "unknown download manager '{name}' (try: {})",
            dm::available_names().join(", ")
        )));
    }
    crate::core::settings::apply(ctx, "dm", name)?;
    println!("active download manager: {name}");
    Ok(ExitCode::Ok)
}

/// `dm` — list detected managers and the active setting.
fn list(ctx: &Context) -> Result<ExitCode, Error> {
    let active = dm::active_name(ctx)?;
    println!("active: {active}");
    for adapter in dm::registry() {
        let path = adapter
            .detect()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "not detected".to_string());
        println!("{}: {}", adapter.name(), path);
    }
    Ok(ExitCode::Ok)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::settings::Settings;

    fn tmp_ctx(tag: &str) -> Context {
        let dir = std::env::temp_dir().join(format!("lewdzone-dm-{tag}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        Context::new(dir.join("catalog.db"), dir.join("config.json"))
    }

    fn remove_ctx(dir_tag: &str) {
        let dir =
            std::env::temp_dir().join(format!("lewdzone-dm-{dir_tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn activate_unknown_manager_is_usage_error() {
        let ctx = tmp_ctx("activate-unknown");
        let err = run(&ctx, Some("not-a-manager")).unwrap_err();
        assert!(matches!(err, Error::Usage(_)));
        remove_ctx("activate-unknown");
    }

    #[test]
    fn activate_persists_dm_setting() {
        let name = dm::available_names().first().copied().unwrap_or("fdm");
        let ctx = tmp_ctx("activate-ok");
        assert!(run(&ctx, Some(name)).is_ok());
        let s = Settings::load(&ctx.config_path).unwrap();
        assert_eq!(
            s.get_value("dm")
                .and_then(|v| v.as_str().map(str::to_string)),
            Some(name.to_string())
        );
        remove_ctx("activate-ok");
    }

    #[test]
    fn list_always_succeeds_with_active() {
        let ctx = tmp_ctx("list");
        assert!(run(&ctx, None).is_ok());
        remove_ctx("list");
    }
}
