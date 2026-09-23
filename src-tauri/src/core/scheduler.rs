//! `scheduler` — serialize downloads so cloud free-tier providers aren't
//! hammered (owner: download family).
//!
//! The site's mirrors (Mega, Google Drive, MediaFire, ...) throttle or block
//! free-tier accounts when many downloads start at once. `download` therefore
//! dispatches jobs ONE at a time with a configurable grace period carved
//! between each start (`download-grace-seconds`, default 20). Sleep is an
//! injected seam so the pacing logic is testable offline with zero real delay
//! (Rule 11).

use std::time::Duration;

use crate::core::download::Job;
use crate::core::{Context, Error};

/// Default pause between consecutive download starts (seconds).
pub const DEFAULT_GRACE_SECONDS: u64 = 20;

/// Serialize dispatch: one job at a time, waiting `grace` between each start.
/// `dispatch` fires per job (in-app stream or OS handler); `sleep` is injected
/// so tests can record the pacing without actually waiting.
pub fn paced(
    grace: Duration,
    jobs: &[Job],
    dispatch: &mut dyn FnMut(&Job) -> Result<(), Error>,
    sleep: &mut dyn FnMut(Duration),
) -> Result<(), Error> {
    for (i, job) in jobs.iter().enumerate() {
        if i > 0 {
            sleep(grace);
        }
        dispatch(job)?;
    }
    Ok(())
}

/// The grace period for a context: the `download-grace-seconds` setting when
/// present, otherwise `DEFAULT_GRACE_SECONDS`.
pub fn grace_for(ctx: &Context) -> Duration {
    let s = crate::core::settings::Settings::load(&ctx.config_path).unwrap_or_default();
    let secs = s
        .download_grace_seconds
        .filter(|s| *s > 0)
        .unwrap_or(DEFAULT_GRACE_SECONDS);
    Duration::from_secs(secs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn job(n: usize, url: &str) -> Job {
        Job {
            game: "wild-life".to_string(),
            title: "Wild Life".to_string(),
            post_id: Some(n as i64),
            engine: None,
            version: "v1.0".to_string(),
            platform: "pc".to_string(),
            tab: "official".to_string(),
            url: url.to_string(),
            target: Some(PathBuf::from(format!("test-dl/{n}.zip"))),
            install_dir: Some(PathBuf::from(format!("test-lzapps/wild-life-{n}"))),
        }
    }

    fn mem_ctx() -> Context {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        Context::new(
            std::env::temp_dir().join(format!("lz-sched-{unique}.db")),
            std::env::temp_dir().join(format!("lz-sched-{unique}-config.json")),
        )
    }

    #[test]
    fn paced_dispatches_one_at_a_time_in_order() {
        let jobs = vec![job(1, "a"), job(2, "b"), job(3, "c")];
        let mut order: Vec<String> = Vec::new();
        let mut slept: Vec<Duration> = Vec::new();
        let _code = paced(
            Duration::from_secs(2),
            &jobs,
            &mut |j| {
                order.push(j.url.clone());
                Ok(())
            },
            &mut |d| slept.push(d),
        )
        .unwrap();
        assert_eq!(order, vec!["a", "b", "c"]);
        assert_eq!(slept.len(), 2, "grace between jobs, none before first");
    }

    #[test]
    fn paced_single_job_sleeps_nothing() {
        let jobs = vec![job(1, "a")];
        let mut slept: Vec<Duration> = Vec::new();
        paced(Duration::from_secs(9), &jobs, &mut |_| Ok(()), &mut |d| {
            slept.push(d)
        })
        .unwrap();
        assert!(slept.is_empty());
    }

    #[test]
    fn paced_stops_on_first_dispatch_error() {
        let jobs = vec![job(1, "a"), job(2, "b")];
        let mut calls = 0;
        let err = paced(
            Duration::ZERO,
            &jobs,
            &mut |_| {
                calls += 1;
                if calls > 1 {
                    Err(Error::Usage("boom".to_string()))
                } else {
                    Ok(())
                }
            },
            &mut |_| {},
        )
        .unwrap_err();
        assert!(matches!(err, Error::Usage(_)));
        assert_eq!(calls, 2);
    }

    #[test]
    fn grace_for_defaults_when_unset() {
        let ctx = mem_ctx();
        // No config file → defaults hold.
        assert_eq!(grace_for(&ctx), Duration::from_secs(DEFAULT_GRACE_SECONDS));
    }

    #[test]
    fn grace_for_reads_the_setting_when_present() {
        let ctx = mem_ctx();
        crate::core::settings::Settings {
            download_grace_seconds: Some(5),
            ..Default::default()
        }
        .save(&ctx.config_path)
        .unwrap();
        assert_eq!(grace_for(&ctx), Duration::from_secs(5));
    }

    #[test]
    fn grace_for_ignores_zero_and_negative() {
        let ctx = mem_ctx();
        crate::core::settings::Settings {
            download_grace_seconds: Some(0),
            ..Default::default()
        }
        .save(&ctx.config_path)
        .unwrap();
        assert_eq!(grace_for(&ctx), Duration::from_secs(DEFAULT_GRACE_SECONDS));
    }
}
