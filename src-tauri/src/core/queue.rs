//! `queue` — non-blocking download queue + worker thread (owner: dm family,
//! dispatch skill).
//!
//! The Storefront's Download button must never block the webview on network
//! (Rule 05) or on the resolver's start→reveal rounds — that froze the app and
//! left users unable to schedule further downloads. Instead, the webview
//! enqueues a request and returns instantly; a background worker picks queued
//! requests up one at a time, resolves their go-links, and dispatches (paced,
//! Rule 05-style throttle protection) to the active manager. Every state
//! change is recorded so the Downloads view can poll [`Queue::snapshot`].
//!
//! The CLI stays fully synchronous (one shot per process) and does NOT use the
//! queue — both entry points share the same `core::download` pipeline, which is
//! what Rule 13 requires (one core, two entry points).

use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::core::download::{self, Job, Select};
use crate::core::{Context, Error};
use crate::resolver;

/// Maximum number of queued requests kept in memory (newest-first view).
const MAX_QUEUED: usize = 64;

/// One enqueued download request's lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// Waiting for the worker to pick it up.
    Queued,
    /// Fetching the game page + resolving go-link tokens.
    Resolving,
    /// Handing resolved URLs to the manager(s).
    Dispatching,
    /// All jobs handed off successfully.
    Dispatched,
    /// Resolution or dispatch failed (see `message`).
    Failed,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Queued => write!(f, "queued"),
            Self::Resolving => write!(f, "resolving"),
            Self::Dispatching => write!(f, "dispatching"),
            Self::Dispatched => write!(f, "dispatched"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// A serializable view of one queued download (what the Downloads page shows).
#[derive(Debug, Clone, serde::Serialize)]
pub struct QueueJob {
    pub id: u64,
    pub slug: String,
    pub version: String,
    pub platform: String,
    pub tab: String,
    pub source: Option<String>,
    pub status: Status,
    pub message: Option<String>,
    pub manager: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Default)]
struct Inner {
    jobs: Vec<QueueJob>,
    next_id: u64,
}

/// Shared, thread-safe request queue + Condvar so the worker sleeps between
/// picks instead of polling.
#[derive(Default)]
pub struct Queue {
    inner: Mutex<Inner>,
    wake: Condvar,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl Queue {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Inner {
                jobs: Vec::new(),
                next_id: 1,
            }),
            wake: Condvar::new(),
        }
    }

    /// Add a request to the back of the queue and wake the worker.
    /// Returns the queued job view. Fails with a usage error when full.
    pub fn enqueue(
        &self,
        slug: String,
        version: String,
        platform: String,
        tab: String,
        source: Option<String>,
    ) -> Result<QueueJob, Error> {
        let mut g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        if g.jobs.len() >= MAX_QUEUED {
            return Err(Error::Usage(
                "download queue is full — wait for active downloads to finish".to_string(),
            ));
        }
        let id = g.next_id;
        g.next_id += 1;
        let now = now_secs();
        let job = QueueJob {
            id,
            slug,
            version,
            platform,
            tab,
            source,
            status: Status::Queued,
            message: None,
            manager: None,
            created_at: now,
            updated_at: now,
        };
        g.jobs.push(job.clone());
        self.wake.notify_one();
        Ok(job)
    }

    /// All jobs, newest first (the Downloads page's view).
    pub fn snapshot(&self) -> Vec<QueueJob> {
        let g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        let mut jobs = g.jobs.clone();
        jobs.reverse();
        jobs
    }

    /// The job with `id`, if it exists.
    pub fn get(&self, id: u64) -> Option<QueueJob> {
        let g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        g.jobs.iter().find(|j| j.id == id).cloned()
    }

    /// Mutate one job under the lock and bump its `updated_at`.
    pub fn update(&self, id: u64, f: impl FnOnce(&mut QueueJob)) {
        let mut g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(job) = g.jobs.iter_mut().find(|j| j.id == id) {
            f(job);
            job.updated_at = now_secs();
        }
    }

    /// Block until a request is `Queued`, then return its id (worker loop).
    fn wait_for_queued(&self) -> u64 {
        let mut g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        loop {
            if let Some(id) = g
                .jobs
                .iter()
                .find(|j| j.status == Status::Queued)
                .map(|j| j.id)
            {
                return id;
            }
            g = self.wake.wait(g).unwrap_or_else(|p| p.into_inner());
        }
    }
}

/// Spawn the background worker that drains the queue. Runs for the life of the
/// process; the runtime context is owned (not borrowed) so no locks are held
/// while network/resolver/manager work happens.
pub fn spawn_worker(queue: Arc<Queue>, ctx: Context) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || loop {
        let id = queue.wait_for_queued();
        let grace = crate::core::scheduler::grace_for(&ctx);
        process(
            &queue,
            &ctx,
            id,
            &mut |url| crate::scraper::fetch(url),
            &mut |go| resolver::resolve(go),
            &mut |job| download::dispatch(&ctx, job),
            grace,
            &mut |d| std::thread::sleep(d),
        );
    })
}

/// Resolve + dispatch ONE queued request, recording progress/result on the
/// queue. Injectable seams (`fetch`, `resolve`, `dispatch`, `sleep`) keep every
/// path offline-testable with fixtures (Rule 11). The worker thread passes the
/// real network/manager seams; tests pass stubs.
#[allow(clippy::too_many_arguments)]
pub fn process(
    queue: &Queue,
    ctx: &Context,
    id: u64,
    fetch: &mut dyn FnMut(&str) -> Result<String, Error>,
    resolve: &mut dyn FnMut(&str) -> Result<resolver::ResolvedUrl, Error>,
    dispatch: &mut dyn FnMut(&Job) -> Result<(), Error>,
    grace: Duration,
    sleep: &mut dyn FnMut(Duration),
) {
    let entry = queue.get(id);
    let Some(entry) = entry else {
        return;
    };
    if entry.status != Status::Queued {
        return;
    }
    queue.update(id, |j| {
        j.status = Status::Resolving;
        j.message = None;
    });

    let select = Select {
        game: &entry.slug,
        version: &entry.version,
        platform: &entry.platform,
        tab: &entry.tab,
        source: entry.source.as_deref(),
    };

    let jobs = match download::jobs_for(ctx, &select, fetch, resolve) {
        Ok(jobs) => jobs,
        Err(err) => {
            queue.update(id, |j| {
                j.status = Status::Failed;
                j.message = Some(err.to_string());
            });
            return;
        }
    };

    if jobs.is_empty() {
        queue.update(id, |j| {
            j.status = Status::Failed;
            j.message = Some("no matching download entries found".to_string());
        });
        return;
    }

    queue.update(id, |j| {
        j.status = Status::Dispatching;
        j.manager = Some(jobs[0].manager.clone());
    });

    match crate::core::scheduler::paced(grace, &jobs, dispatch, sleep) {
        Ok(()) => queue.update(id, |j| {
            j.status = Status::Dispatched;
            j.message = Some(format!(
                "{} job(s) handed to {}",
                jobs.len(),
                jobs[0].manager
            ));
        }),
        Err(err) => queue.update(id, |j| {
            j.status = Status::Failed;
            j.message = Some(err.to_string());
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GAME_FIXTURE: &str = include_str!("../../tests/fixtures/html/lz_game.html");

    fn mem_ctx() -> Context {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        Context::new(
            std::env::temp_dir().join(format!("lz-queue-{unique}.db")),
            std::env::temp_dir().join(format!("lz-queue-{unique}-config.json")),
        )
    }

    fn stub_resolve(url: &str) -> Result<resolver::ResolvedUrl, Error> {
        Ok(resolver::ResolvedUrl {
            url: format!("https://fileknot.io/dl/{}", url),
            host: "fileknot".to_string(),
            platform: Some("pc".to_string()),
            version: None,
            game_id: None,
        })
    }

    fn enqueue_sample(q: &Queue) -> u64 {
        q.enqueue(
            "treasure-of-nadia".to_string(),
            "latest".to_string(),
            "PC".to_string(),
            "official".to_string(),
            None,
        )
        .unwrap()
        .id
    }

    #[test]
    fn enqueue_assigns_ids_and_snapshot_is_newest_first() {
        let q = Queue::new();
        let a = q
            .enqueue(
                "a".into(),
                "latest".into(),
                "PC".into(),
                "official".into(),
                None,
            )
            .unwrap();
        let b = q
            .enqueue(
                "b".into(),
                "latest".into(),
                "PC".into(),
                "official".into(),
                None,
            )
            .unwrap();
        assert_eq!(a.id, 1);
        assert_eq!(b.id, 2);
        assert_eq!(a.status, Status::Queued);
        let snap = q.snapshot();
        assert_eq!(snap.len(), 2);
        assert_eq!(snap[0].id, 2, "newest first");
        assert_eq!(snap[1].id, 1);
    }

    #[test]
    fn enqueue_rejects_when_full() {
        let q = Queue::new();
        for _ in 0..MAX_QUEUED {
            q.enqueue(
                "x".into(),
                "latest".into(),
                "PC".into(),
                "official".into(),
                None,
            )
            .unwrap();
        }
        let err = q
            .enqueue(
                "y".into(),
                "latest".into(),
                "PC".into(),
                "official".into(),
                None,
            )
            .unwrap_err();
        assert!(matches!(err, Error::Usage(_)));
    }

    #[test]
    fn update_changes_status_and_bumps_timestamp() {
        let q = Queue::new();
        let id = enqueue_sample(&q);
        q.update(id, |j| j.status = Status::Resolving);
        let job = q.get(id).unwrap();
        assert_eq!(job.status, Status::Resolving);
        assert!(job.updated_at >= job.created_at);
    }

    #[test]
    fn process_resolves_and_dispatches_all_jobs_offline() {
        let q = Queue::new();
        let id = enqueue_sample(&q);
        let mut dispatched: Vec<String> = Vec::new();
        process(
            &q,
            &mem_ctx(),
            id,
            &mut |url| {
                assert!(url.ends_with("//lewdzone.com/game/treasure-of-nadia/"));
                Ok(GAME_FIXTURE.to_string())
            },
            &mut stub_resolve,
            &mut |job| {
                dispatched.push(job.url.clone());
                let _ =
                    std::fs::create_dir_all(job.target.as_ref().and_then(|p| p.parent()).unwrap());
                Ok(())
            },
            Duration::ZERO,
            &mut |_| {},
        );
        assert!(!dispatched.is_empty(), "fixture yields multiple jobs");
        let job = q.get(id).unwrap();
        assert_eq!(job.status, Status::Dispatched);
        assert!(job.manager.is_some());
        let msg = job.message.unwrap();
        assert!(msg.contains("job(s) handed to"), "msg was: {msg}");
    }

    #[test]
    fn process_marks_failed_on_network_error() {
        let q = Queue::new();
        let id = enqueue_sample(&q);
        process(
            &q,
            &mem_ctx(),
            id,
            &mut |_| Err(Error::Network("offline".to_string())),
            &mut stub_resolve,
            &mut |_| Ok(()),
            Duration::ZERO,
            &mut |_| {},
        );
        let job = q.get(id).unwrap();
        assert_eq!(job.status, Status::Failed);
        assert!(job.message.unwrap().contains("network error"));
    }

    #[test]
    fn process_marks_failed_on_dispatch_error() {
        let q = Queue::new();
        let id = enqueue_sample(&q);
        process(
            &q,
            &mem_ctx(),
            id,
            &mut |_| Ok(GAME_FIXTURE.to_string()),
            &mut stub_resolve,
            &mut |_| Err(Error::Runtime("manager exploded".to_string())),
            Duration::ZERO,
            &mut |_| {},
        );
        let job = q.get(id).unwrap();
        assert_eq!(job.status, Status::Failed);
        assert!(job.message.unwrap().contains("manager exploded"));
    }

    #[test]
    fn process_respects_queued_guard_and_pacing_sleeps() {
        let q = Queue::new();
        let id = enqueue_sample(&q);
        // Mark dispatched first: worker must NOT re-run a finished request.
        q.update(id, |j| j.status = Status::Dispatched);
        let mut calls = 0;
        let mut slept = 0usize;
        process(
            &q,
            &mem_ctx(),
            id,
            &mut |_| Ok(GAME_FIXTURE.to_string()),
            &mut stub_resolve,
            &mut |_| {
                calls += 1;
                Ok(())
            },
            Duration::from_secs(2),
            &mut |_| slept += 1,
        );
        assert_eq!(calls, 0, "already-dispatched request is skipped");
        assert_eq!(slept, 0);
    }
}
