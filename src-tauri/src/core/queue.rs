//! `queue` — non-blocking download queue + worker thread (owner: download
//! family, dispatch skill).
//!
//! The Storefront's Download button must never block the webview on network
//! (Rule 05) or on the resolver's start→reveal rounds — that froze the app and
//! left users unable to schedule further downloads. Instead, the webview
//! enqueues a request and returns instantly; a background worker picks queued
//! requests up one at a time, resolves their go-links, and dispatches (paced,
//! Rule 05-style throttle protection): direct-file hosts are streamed in-app
//! with byte progress, everything else opens in the OS default handler. Every
//! state change is recorded so the Downloads view can poll [`Queue::snapshot`].
//!
//! The CLI stays fully synchronous (one shot per process) and does NOT use the
//! queue — both entry points share the same `core::download` pipeline, which is
//! what Rule 13 requires (one core, two entry points).

use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::core::download::{self, Job, Select};
use crate::core::{Context, Error};
use crate::db;
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
    /// Dispatching jobs (paced) to the OS default handler or a stream.
    Dispatching,
    /// Streaming a direct-file host into the download root (bytes shown live).
    Downloading,
    /// All jobs handed off successfully.
    Dispatched,
    /// Resolution or dispatch failed (see `message`).
    Failed,
}

impl Status {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Resolving => "resolving",
            Self::Dispatching => "dispatching",
            Self::Downloading => "downloading",
            Self::Dispatched => "dispatched",
            Self::Failed => "failed",
        }
    }
}

impl std::str::FromStr for Status {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "queued" => Ok(Self::Queued),
            "resolving" => Ok(Self::Resolving),
            "dispatching" => Ok(Self::Dispatching),
            "downloading" => Ok(Self::Downloading),
            "dispatched" => Ok(Self::Dispatched),
            "failed" => Ok(Self::Failed),
            other => Err(Error::Runtime(format!("unknown queue status: {other}"))),
        }
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
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
    /// Bytes streamed so far / total for direct-file hosts (0 when unknown or
    /// when the job opens in the OS handler instead of streaming).
    pub bytes_done: u64,
    pub bytes_total: u64,
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
    /// When set, every enqueue/update is mirrored to the SQLite `queue_job`
    /// table so queued jobs survive restarts.
    db_path: Option<PathBuf>,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl Queue {
    /// In-memory queue for tests and CLI one-shots.
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Inner {
                jobs: Vec::new(),
                next_id: 1,
            }),
            wake: Condvar::new(),
            db_path: None,
        }
    }

    /// Load persisted queued jobs from the database and resume the worker.
    /// This is the queue the GUI uses across restarts.
    pub fn load(ctx: &Context) -> Result<Self, Error> {
        let conn = db::open(&ctx.db_path)?;
        db::migrate(&conn)?;
        let rows = crate::db::repo::queue_load_all(&conn)?;
        let mut next_id = rows.iter().map(|r| r.id).max().unwrap_or(0);
        next_id += 1;
        let jobs: Vec<QueueJob> = rows
            .into_iter()
            .map(|r| QueueJob {
                id: r.id,
                slug: r.slug,
                version: r.version,
                platform: r.platform,
                tab: r.tab,
                source: r.source,
                status: r.status.parse().unwrap_or(Status::Queued),
                message: r.message,
                bytes_done: r.bytes_done,
                bytes_total: r.bytes_total,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();
        Ok(Self {
            inner: Mutex::new(Inner { jobs, next_id }),
            wake: Condvar::new(),
            db_path: Some(ctx.db_path.clone()),
        })
    }

    fn persist(&self, job: &QueueJob) -> Result<(), Error> {
        let Some(path) = &self.db_path else {
            return Ok(());
        };
        let conn = db::open(path)?;
        let tx = conn.unchecked_transaction()?;
        let row = crate::db::repo::QueueJobRow {
            id: job.id,
            slug: job.slug.clone(),
            version: job.version.clone(),
            platform: job.platform.clone(),
            tab: job.tab.clone(),
            source: job.source.clone(),
            status: job.status.as_str().to_string(),
            message: job.message.clone(),
            bytes_done: job.bytes_done,
            bytes_total: job.bytes_total,
            created_at: job.created_at,
            updated_at: job.updated_at,
        };
        crate::db::repo::queue_upsert(&tx, &row)?;
        tx.commit()?;
        Ok(())
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
            bytes_done: 0,
            bytes_total: 0,
            created_at: now,
            updated_at: now,
        };
        g.jobs.push(job.clone());
        drop(g);
        self.persist(&job)?;
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

    /// Mutate one job under the lock, bump its `updated_at`, and mirror the
    /// change to SQLite when persistence is enabled.
    pub fn update(&self, id: u64, f: impl FnOnce(&mut QueueJob)) {
        let mut g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(job) = g.jobs.iter_mut().find(|j| j.id == id) {
            f(job);
            job.updated_at = now_secs();
            let job = job.clone();
            drop(g);
            let _ = self.persist(&job);
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
/// while network/resolver/dispatch work happens.
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
            &mut |job, progress| {
                download::dispatch_with(job, &mut crate::scraper::download_stream, progress)
            },
            grace,
            &mut |d| std::thread::sleep(d),
        );
    })
}

/// Resolve + dispatch ONE queued request, recording progress/result on the
/// queue. Injectable seams (`fetch`, `resolve`, `dispatch`, `sleep`) keep every
/// path offline-testable with fixtures (Rule 11). The worker thread passes the
/// real network/dispatch seams; tests pass stubs. The `dispatch` seam reports
/// `(bytes done, total)` so direct-host streams drive the `Downloading` state
/// and the progress bar; OS-handler jobs never call it and stay `Dispatching`.
#[allow(clippy::too_many_arguments)]
pub fn process(
    queue: &Queue,
    ctx: &Context,
    id: u64,
    fetch: &mut dyn FnMut(&str) -> Result<String, Error>,
    resolve: &mut dyn FnMut(&str) -> Result<resolver::ResolvedUrl, Error>,
    dispatch: &mut dyn FnMut(&Job, download::ProgressCallback<'_>) -> Result<(), Error>,
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
        j.bytes_done = 0;
        j.bytes_total = 0;
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
    });

    let streamed = jobs
        .iter()
        .filter(|j| download::is_direct_stream_host(&j.tab))
        .count();
    let opened = jobs.len() - streamed;
    let mut dispatch_adapt = |job: &Job| {
        queue.update(id, |j| {
            j.status = Status::Dispatching;
        });
        let mut progress = |done: u64, total: u64| {
            queue.update(id, |j| {
                j.status = Status::Downloading;
                j.bytes_done = done;
                j.bytes_total = total;
            });
        };
        dispatch(job, &mut progress)
    };

    match crate::core::scheduler::paced(grace, &jobs, &mut dispatch_adapt, sleep) {
        Ok(()) => queue.update(id, |j| {
            j.status = Status::Dispatched;
            let (done, total) = (j.bytes_done, j.bytes_total);
            let suffix = if opened > 0 {
                format!(", {opened} opened")
            } else {
                String::new()
            };
            j.message = Some(if total > 0 {
                format!("downloaded {done} of {total} bytes{suffix}")
            } else if done > 0 {
                format!("downloaded {done} bytes{suffix}")
            } else if opened > 0 {
                format!("{opened} job(s) opened in the default handler")
            } else {
                format!("{streamed} job(s) dispatched")
            });
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
            &mut |job, _progress| {
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
        let msg = job.message.unwrap();
        assert!(msg.contains("job(s)"), "msg was: {msg}");
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
            &mut |_, _progress| Ok(()),
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
            &mut |_, _| Err(Error::Runtime("dispatch exploded".to_string())),
            Duration::ZERO,
            &mut |_| {},
        );
        let job = q.get(id).unwrap();
        assert_eq!(job.status, Status::Failed);
        assert!(job.message.unwrap().contains("dispatch exploded"));
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
            &mut |_, _| {
                calls += 1;
                Ok(())
            },
            Duration::from_secs(2),
            &mut |_| slept += 1,
        );
        assert_eq!(calls, 0, "already-dispatched request is skipped");
        assert_eq!(slept, 0);
    }

    #[test]
    fn process_records_stream_progress_bytes() {
        let q = Queue::new();
        let id = enqueue_sample(&q);
        process(
            &q,
            &mem_ctx(),
            id,
            &mut |_| Ok(GAME_FIXTURE.to_string()),
            &mut stub_resolve,
            &mut |_job, progress| {
                progress(3, 10);
                progress(10, 10);
                Ok(())
            },
            Duration::ZERO,
            &mut |_| {},
        );
        let job = q.get(id).unwrap();
        assert_eq!(job.status, Status::Dispatched);
        assert_eq!(job.bytes_done, 10);
        assert_eq!(job.bytes_total, 10);
        assert!(job.message.unwrap().contains("bytes"));
    }

    #[test]
    fn queue_persists_queued_jobs_and_resumes_them() {
        let ctx = mem_ctx();
        let q = Queue::load(&ctx).unwrap();
        let id = q
            .enqueue(
                "treasure-of-nadia".into(),
                "latest".into(),
                "PC".into(),
                "official".into(),
                Some("fileknot".into()),
            )
            .unwrap()
            .id;
        assert_eq!(id, 1);

        // Simulate a restart: load a fresh Queue from the same DB.
        let q2 = Queue::load(&ctx).unwrap();
        let jobs = q2.snapshot();
        assert_eq!(jobs.len(), 1);
        let job = &jobs[0];
        assert_eq!(job.slug, "treasure-of-nadia");
        assert_eq!(job.version, "latest");
        assert_eq!(job.platform, "PC");
        assert_eq!(job.tab, "official");
        assert_eq!(job.source.as_deref(), Some("fileknot"));
        assert_eq!(job.status, Status::Queued);
        assert_eq!(job.id, 1);

        // State updates must also persist.
        q2.update(id, |j| j.status = Status::Resolving);
        let q3 = Queue::load(&ctx).unwrap();
        assert_eq!(q3.get(id).unwrap().status, Status::Resolving);
    }
}
