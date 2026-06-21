use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, UNIX_EPOCH};

use autoflow_application::blueprints;
use autoflow_application::ports::JobStore;
use autoflow_application::BlueprintStore;
use autoflow_domain::{Blueprint, DomainError, Job, WatchConfig, WatchEventKind};
use autoflow_infrastructure::audit::SqliteAuditStore;
use autoflow_infrastructure::blueprints::SqliteBlueprintStore;
use autoflow_infrastructure::jobs::SqliteJobStore;
use notify::event::ModifyKind;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, FileIdMap};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::queue::JobQueue;

type FullDebouncer = Debouncer<RecommendedWatcher, FileIdMap>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WatchTarget {
    Job(Uuid),
    Blueprint(Uuid),
}

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
    blueprints: Arc<SqliteBlueprintStore>,
    handles: Mutex<HashMap<WatchTarget, WatchHandle>>,
    trigger_tx: mpsc::UnboundedSender<WatchTarget>,
}

impl WatchService {
    pub fn start(
        jobs: Arc<SqliteJobStore>,
        blueprints: Arc<SqliteBlueprintStore>,
        audit: Arc<SqliteAuditStore>,
        queue: JobQueue,
    ) -> Self {
        let (trigger_tx, mut trigger_rx) = mpsc::unbounded_channel();
        let queue_worker = queue.clone();
        let blueprints_worker = blueprints.clone();
        let audit_worker = audit.clone();

        tokio::spawn(async move {
            while let Some(target) = trigger_rx.recv().await {
                match target {
                    WatchTarget::Job(job_id) => {
                        if let Err(error) = queue_worker.enqueue(job_id) {
                            if !matches!(error, DomainError::AlreadyRunning) {
                                tracing::warn!(%job_id, %error, "watch failed to enqueue job");
                            }
                        }
                    }
                    WatchTarget::Blueprint(blueprint_id) => {
                        let store = blueprints_worker.clone();
                        let audit = audit_worker.clone();
                        tokio::spawn(async move {
                            if let Err(error) = blueprints::apply_blueprint(
                                store.as_ref(),
                                audit.as_ref(),
                                blueprint_id,
                            )
                            .await
                            {
                                tracing::warn!(%blueprint_id, %error, "watch failed to apply blueprint");
                            }
                        });
                    }
                }
            }
        });

        let service = Self {
            inner: Arc::new(WatchServiceInner {
                jobs: jobs.clone(),
                blueprints: blueprints.clone(),
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

        let blueprint_summaries = self.inner.blueprints.list().await?;
        for summary in blueprint_summaries {
            let blueprint = self.inner.blueprints.get(summary.id).await?;
            self.sync_blueprint(&blueprint).await?;
        }

        Ok(())
    }

    pub async fn sync_job(&self, job: &Job) -> Result<(), DomainError> {
        let target = WatchTarget::Job(job.id);
        self.unregister(target);

        if !should_watch_job(job) {
            return Ok(());
        }

        let watch = job.watch.as_ref().expect("watch config required");
        self.register(target, watch, &job.source_path, job.filters.recursive)?;
        Ok(())
    }

    pub async fn sync_blueprint(&self, blueprint: &Blueprint) -> Result<(), DomainError> {
        let target = WatchTarget::Blueprint(blueprint.id);
        self.unregister(target);

        if !should_watch_blueprint(blueprint) {
            return Ok(());
        }

        let watch = blueprint.watch.as_ref().expect("watch config required");
        self.register(target, watch, &blueprint.root_path, blueprint.recursive)?;
        Ok(())
    }

    pub fn unregister(&self, target: WatchTarget) {
        self.inner.handles.lock().unwrap().remove(&target);
    }

    pub fn unregister_job(&self, job_id: Uuid) {
        self.unregister(WatchTarget::Job(job_id));
    }

    pub fn unregister_blueprint(&self, blueprint_id: Uuid) {
        self.unregister(WatchTarget::Blueprint(blueprint_id));
    }

    fn register(
        &self,
        target: WatchTarget,
        watch: &WatchConfig,
        root_path: &str,
        recursive: bool,
    ) -> Result<(), DomainError> {
        let source = PathBuf::from(root_path);
        if !source.exists() {
            return Err(DomainError::SourceNotFound(root_path.to_string()));
        }

        let mut debouncer = None;
        let mut poll_task = None;

        if watch.uses_realtime() {
            debouncer = Some(self.start_realtime(target, watch, &source, recursive)?);
        }

        if watch.uses_polling() {
            let interval_secs = watch
                .poll_interval_secs()
                .expect("polling interval required");
            poll_task = Some(self.start_polling(target, &source, interval_secs));
        }

        self.inner.handles.lock().unwrap().insert(
            target,
            WatchHandle {
                _debouncer: debouncer,
                _poll_task: poll_task,
            },
        );

        Ok(())
    }

    fn start_realtime(
        &self,
        target: WatchTarget,
        watch: &WatchConfig,
        source: &Path,
        recursive: bool,
    ) -> Result<FullDebouncer, DomainError> {
        let allowed = watch.events.clone();
        let tx = self.inner.trigger_tx.clone();
        let settle = Duration::from_secs(watch.settle_seconds as u64);

        let mut debouncer = new_debouncer(settle, None, move |result: DebounceEventResult| {
            let Ok(events) = result else {
                return;
            };
            for debounced in events {
                if matches_watch_event(&debounced.event.kind, &allowed) {
                    let _ = tx.send(target);
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

    fn start_polling(
        &self,
        target: WatchTarget,
        source: &Path,
        interval_secs: u32,
    ) -> JoinHandle<()> {
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
                    let _ = tx.send(target);
                }
            }
        })
    }
}

fn should_watch_job(job: &Job) -> bool {
    job.enabled
        && job.watch.as_ref().is_some_and(|watch| watch.enabled)
        && Path::new(&job.source_path).exists()
}

fn should_watch_blueprint(blueprint: &Blueprint) -> bool {
    blueprint.enabled
        && blueprint.watch.as_ref().is_some_and(|watch| watch.enabled)
        && Path::new(&blueprint.root_path).exists()
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
