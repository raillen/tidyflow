use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, UNIX_EPOCH};

use autoflow_application::ports::JobStore;
use autoflow_domain::{DomainError, Job, WatchConfig, WatchEventKind};
use autoflow_infrastructure::jobs::SqliteJobStore;
use notify::event::ModifyKind;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, FileIdMap};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::queue::JobQueue;

type FullDebouncer = Debouncer<RecommendedWatcher, FileIdMap>;

struct WatchHandle {
    _debouncer: Option<FullDebouncer>,
    _poll_task: Option<JoinHandle<()>>,
}

#[derive(Clone)]
pub struct WatchService {
    inner: Arc<WatchServiceInner>,
}

struct WatchServiceInner {
    jobs: Arc<SqliteJobStore>,
    handles: Mutex<HashMap<Uuid, WatchHandle>>,
    trigger_tx: mpsc::UnboundedSender<Uuid>,
}

impl WatchService {
    pub fn start(jobs: Arc<SqliteJobStore>, queue: JobQueue) -> Self {
        let (trigger_tx, mut trigger_rx) = mpsc::unbounded_channel();
        let queue_worker = queue.clone();
        tokio::spawn(async move {
            while let Some(job_id) = trigger_rx.recv().await {
                if let Err(error) = queue_worker.enqueue(job_id) {
                    if !matches!(error, DomainError::AlreadyRunning) {
                        tracing::warn!(%job_id, %error, "watch failed to enqueue job");
                    }
                }
            }
        });

        let service = Self {
            inner: Arc::new(WatchServiceInner {
                jobs: jobs.clone(),
                handles: Mutex::new(HashMap::new()),
                trigger_tx,
            }),
        };

        let boot = service.clone();
        tokio::spawn(async move {
            if let Err(error) = boot.sync_all().await {
                tracing::warn!(%error, "watch sync_all failed on startup");
            }
        });

        service
    }

    pub async fn sync_all(&self) -> Result<(), DomainError> {
        let summaries = self.inner.jobs.list().await?;
        for summary in summaries {
            let job = self.inner.jobs.get(summary.id).await?;
            self.sync_job(&job).await?;
        }
        Ok(())
    }

    pub async fn sync_job(&self, job: &Job) -> Result<(), DomainError> {
        self.unregister(job.id);

        if !should_watch(job) {
            return Ok(());
        }

        let watch = job.watch.as_ref().expect("watch config required");
        self.register(job, watch)?;
        Ok(())
    }

    pub fn unregister(&self, job_id: Uuid) {
        self.inner.handles.lock().unwrap().remove(&job_id);
    }

    fn register(&self, job: &Job, watch: &WatchConfig) -> Result<(), DomainError> {
        let source = PathBuf::from(&job.source_path);
        if !source.exists() {
            return Err(DomainError::SourceNotFound(job.source_path.clone()));
        }

        let mut debouncer = None;
        let mut poll_task = None;

        if watch.uses_realtime() {
            debouncer = Some(self.start_realtime(job, watch, &source)?);
        }

        if watch.uses_polling() {
            let interval_secs = watch
                .poll_interval_secs()
                .expect("polling interval required");
            poll_task = Some(self.start_polling(job.id, &source, interval_secs));
        }

        self.inner.handles.lock().unwrap().insert(
            job.id,
            WatchHandle {
                _debouncer: debouncer,
                _poll_task: poll_task,
            },
        );

        Ok(())
    }

    fn start_realtime(
        &self,
        job: &Job,
        watch: &WatchConfig,
        source: &Path,
    ) -> Result<FullDebouncer, DomainError> {
        let job_id = job.id;
        let allowed = watch.events.clone();
        let tx = self.inner.trigger_tx.clone();
        let settle = Duration::from_secs(watch.settle_seconds as u64);
        let recursive = job.filters.recursive;

        let mut debouncer = new_debouncer(settle, None, move |result: DebounceEventResult| {
            let Ok(events) = result else {
                return;
            };
            for debounced in events {
                if matches_watch_event(&debounced.event.kind, &allowed) {
                    let _ = tx.send(job_id);
                    break;
                }
            }
        })
        .map_err(|e| DomainError::Database(format!("watch debouncer: {e}")))?;

        let mode = if recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        debouncer
            .watcher()
            .watch(source, mode)
            .map_err(|e| DomainError::Database(format!("watch path: {e}")))?;

        Ok(debouncer)
    }

    fn start_polling(&self, job_id: Uuid, source: &Path, interval_secs: u32) -> JoinHandle<()> {
        let source = source.to_path_buf();
        let tx = self.inner.trigger_tx.clone();
        let interval = Duration::from_secs(interval_secs as u64);

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            let mut snapshot = directory_snapshot(&source);
            ticker.tick().await;
            loop {
                ticker.tick().await;
                let current = directory_snapshot(&source);
                if snapshot != current {
                    snapshot = current;
                    let _ = tx.send(job_id);
                }
            }
        })
    }
}

fn should_watch(job: &Job) -> bool {
    job.enabled
        && job
            .watch
            .as_ref()
            .is_some_and(|watch| watch.enabled)
        && Path::new(&job.source_path).exists()
}

fn matches_watch_event(kind: &EventKind, allowed: &[WatchEventKind]) -> bool {
    let mapped = match kind {
        EventKind::Create(_) => Some(WatchEventKind::Create),
        EventKind::Modify(ModifyKind::Name(_)) => Some(WatchEventKind::Rename),
        EventKind::Modify(_) => Some(WatchEventKind::Modify),
        EventKind::Remove(_) => Some(WatchEventKind::Remove),
        _ => None,
    };

    mapped.is_some_and(|event| allowed.contains(&event))
}

type DirSnapshot = HashMap<PathBuf, (u64, u64)>;

fn directory_snapshot(root: &Path) -> DirSnapshot {
    let mut snapshot = HashMap::new();
    if !root.exists() {
        return snapshot;
    }

    if root.is_file() {
        if let Some(sig) = file_signature(root) {
            snapshot.insert(root.to_path_buf(), sig);
        }
        return snapshot;
    }

    let walker = walkdir::WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file());

    for entry in walker {
        if let Some(sig) = file_signature(entry.path()) {
            snapshot.insert(entry.path().to_path_buf(), sig);
        }
    }

    snapshot
}

fn file_signature(path: &Path) -> Option<(u64, u64)> {
    let meta = std::fs::metadata(path).ok()?;
    let modified = meta
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    Some((meta.len(), modified))
}
