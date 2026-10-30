use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, UNIX_EPOCH};

use autoflow_application::naming;
use autoflow_application::ports::JobStore;
use autoflow_application::BlueprintStore;
use autoflow_domain::{
    Blueprint, DomainError, Job, NormalizedWatchEvent, WatchConfig, WatchEventKind,
};
use autoflow_infrastructure::audit::SqliteAuditStore;
use autoflow_infrastructure::blueprints::SqliteBlueprintStore;
use autoflow_infrastructure::jobs::SqliteJobStore;
use notify::event::{CreateKind, ModifyKind};
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
    trigger_tx: mpsc::UnboundedSender<NormalizedWatchEvent>,
    blueprint_busy: Mutex<HashSet<Uuid>>,
    blueprint_suppressed_until: Mutex<HashMap<Uuid, Instant>>,
}

impl WatchService {
    pub fn start(
        jobs: Arc<SqliteJobStore>,
        blueprints: Arc<SqliteBlueprintStore>,
        audit: Arc<SqliteAuditStore>,
        queue: JobQueue,
    ) -> Self {
        let (trigger_tx, mut trigger_rx) = mpsc::unbounded_channel();
        let service = Self {
            inner: Arc::new(WatchServiceInner {
                jobs,
                blueprints,
                handles: Mutex::new(HashMap::new()),
                trigger_tx,
                blueprint_busy: Mutex::new(HashSet::new()),
                blueprint_suppressed_until: Mutex::new(HashMap::new()),
            }),
        };

        let worker = service.clone();
        let queue_worker = queue;
        let audit_worker = audit;

        tokio::spawn(async move {
            while let Some(event) = trigger_rx.recv().await {
                match event {
                    NormalizedWatchEvent::JobTriggered { job_id } => {
                        if let Err(error) = queue_worker.enqueue(job_id) {
                            if !matches!(error, DomainError::AlreadyRunning) {
                                tracing::warn!(%job_id, %error, "watch failed to enqueue job");
                            }
                        }
                    }
                    NormalizedWatchEvent::BlueprintTriggered { blueprint_id } => {
                        if worker.is_blueprint_suppressed(blueprint_id)
                            || !worker.begin_blueprint_run(blueprint_id)
                        {
                            tracing::debug!(%blueprint_id, "watch skipped a concurrent or self-triggered blueprint run");
                            continue;
                        }
                        let store = worker.inner.blueprints.clone();
                        let audit = audit_worker.clone();
                        let worker = worker.clone();
                        tokio::spawn(async move {
                            let result = async {
                                let blueprint = store.get(blueprint_id).await?;
                                let variables =
                                    autoflow_application::blueprints::folder_plan_default_variables(
                                        blueprint.folder_plan.as_ref(),
                                    );
                                autoflow_application::blueprints::apply_blueprint_with_variables(
                                    store.as_ref(),
                                    audit.as_ref(),
                                    blueprint_id,
                                    variables,
                                )
                                .await
                            }
                            .await;
                            if let Err(error) = result {
                                tracing::warn!(%blueprint_id, %error, "watch failed to apply blueprint");
                            }
                            worker.end_blueprint_run(blueprint_id);
                        });
                    }
                    NormalizedWatchEvent::FolderCreated {
                        blueprint_id,
                        path,
                        folder_name,
                    } => {
                        tracing::info!(%blueprint_id, path = %path, %folder_name, "received FolderCreated event");
                        if worker.is_blueprint_suppressed(blueprint_id)
                            || !worker.begin_blueprint_run(blueprint_id)
                        {
                            tracing::debug!(%blueprint_id, "watch skipped a concurrent or self-triggered auto-correct run");
                            continue;
                        }
                        let store = worker.inner.blueprints.clone();
                        let audit = audit_worker.clone();
                        let worker = worker.clone();
                        tokio::spawn(async move {
                            let result = naming::handle_folder_auto_correct(
                                store.as_ref(),
                                audit.as_ref(),
                                blueprint_id,
                                &path,
                                &folder_name,
                            )
                            .await;
                            if let Err(error) = result {
                                tracing::warn!(%blueprint_id, %error, "auto-correct failed for folder");
                            }
                            worker.end_blueprint_run(blueprint_id);
                        });
                    }
                }
            }
        });

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
        self.register_job(target, watch, &job.source_path, job.filters.recursive)?;
        Ok(())
    }

    pub async fn sync_blueprint(&self, blueprint: &Blueprint) -> Result<(), DomainError> {
        let target = WatchTarget::Blueprint(blueprint.id);
        self.unregister(target);

        if !should_watch_blueprint(blueprint) {
            return Ok(());
        }

        let watch = blueprint.watch.as_ref().expect("watch config required");
        self.register_blueprint(target, watch, blueprint)?;
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

    fn begin_blueprint_run(&self, blueprint_id: Uuid) -> bool {
        let mut busy = self.inner.blueprint_busy.lock().unwrap();
        if !busy.insert(blueprint_id) {
            return false;
        }
        true
    }

    fn end_blueprint_run(&self, blueprint_id: Uuid) {
        self.inner
            .blueprint_busy
            .lock()
            .unwrap()
            .remove(&blueprint_id);
        self.inner
            .blueprint_suppressed_until
            .lock()
            .unwrap()
            .insert(blueprint_id, Instant::now() + Duration::from_secs(2));
    }

    fn is_blueprint_suppressed(&self, blueprint_id: Uuid) -> bool {
        let mut suppressed = self.inner.blueprint_suppressed_until.lock().unwrap();
        match suppressed.get(&blueprint_id).copied() {
            Some(until) if until > Instant::now() => true,
            Some(_) => {
                suppressed.remove(&blueprint_id);
                false
            }
            None => false,
        }
    }

    fn register_job(
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
            debouncer = Some(self.start_realtime_job(target, watch, &source, recursive)?);
        }

        if watch.uses_polling() {
            let interval_secs = watch
                .poll_interval_secs()
                .expect("polling interval required");
            poll_task = Some(self.start_polling(target, &source, interval_secs, false));
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

    fn register_blueprint(
        &self,
        target: WatchTarget,
        watch: &WatchConfig,
        blueprint: &Blueprint,
    ) -> Result<(), DomainError> {
        let source = PathBuf::from(&blueprint.root_path);
        if !source.exists() {
            return Err(DomainError::SourceNotFound(blueprint.root_path.clone()));
        }

        let mut debouncer = None;
        let mut poll_task = None;

        if watch.uses_realtime() {
            debouncer = Some(self.start_realtime_blueprint(target, watch, blueprint, &source)?);
        }

        if watch.uses_polling() {
            let interval_secs = watch
                .poll_interval_secs()
                .expect("polling interval required");
            poll_task =
                Some(self.start_polling(target, &source, interval_secs, watch.auto_correct));
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

    fn start_realtime_job(
        &self,
        target: WatchTarget,
        watch: &WatchConfig,
        source: &Path,
        recursive: bool,
    ) -> Result<FullDebouncer, DomainError> {
        let allowed = watch.events.clone();
        let tx = self.inner.trigger_tx.clone();
        let settle = Duration::from_secs(watch.settle_seconds as u64);
        let job_id = match target {
            WatchTarget::Job(id) => id,
            WatchTarget::Blueprint(id) => id,
        };

        let mut debouncer = new_debouncer(settle, None, move |result: DebounceEventResult| {
            let Ok(events) = result else {
                return;
            };
            for debounced in events {
                if matches_watch_event(&debounced.event.kind, &allowed) {
                    let _ = tx.send(NormalizedWatchEvent::JobTriggered { job_id });
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

    fn start_realtime_blueprint(
        &self,
        target: WatchTarget,
        watch: &WatchConfig,
        blueprint: &Blueprint,
        source: &Path,
    ) -> Result<FullDebouncer, DomainError> {
        let allowed = watch.events.clone();
        let tx = self.inner.trigger_tx.clone();
        let settle = Duration::from_secs(watch.settle_seconds as u64);
        let auto_correct = watch.auto_correct;
        let recursive = blueprint.recursive;
        let blueprint_id = match target {
            WatchTarget::Blueprint(id) => id,
            WatchTarget::Job(id) => id,
        };
        let root_path = source.to_path_buf();

        let mut debouncer = new_debouncer(settle, None, move |result: DebounceEventResult| {
            let Ok(events) = result else {
                tracing::warn!("debouncer received error result");
                return;
            };
            for debounced in &events {
                tracing::debug!(
                    kind = ?debounced.event.kind,
                    paths = ?debounced.event.paths,
                    "debouncer event received"
                );
                if matches_watch_event(&debounced.event.kind, &allowed) {
                    if auto_correct {
                        if let EventKind::Create(CreateKind::Folder) = &debounced.event.kind {
                            for path in &debounced.event.paths {
                                if !path.starts_with(&root_path) {
                                    continue;
                                }
                                let folder_name = path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("")
                                    .to_string();
                                tracing::info!(%folder_name, "emitting FolderCreated event");
                                let _ = tx.send(NormalizedWatchEvent::FolderCreated {
                                    blueprint_id,
                                    path: path.display().to_string(),
                                    folder_name,
                                });
                                break;
                            }
                        } else {
                            let _ =
                                tx.send(NormalizedWatchEvent::BlueprintTriggered { blueprint_id });
                        }
                    } else {
                        let _ = tx.send(NormalizedWatchEvent::BlueprintTriggered { blueprint_id });
                    }
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
        auto_correct: bool,
    ) -> JoinHandle<()> {
        let root = source.to_path_buf();
        let tx = self.inner.trigger_tx.clone();
        let interval = Duration::from_secs(interval_secs as u64);

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            let mut snapshot = directory_snapshot(&root);
            ticker.tick().await;
            loop {
                ticker.tick().await;
                let current = directory_snapshot(&root);
                if snapshot != current {
                    let changed: Vec<(PathBuf, bool)> = current
                        .iter()
                        .filter(|(path, signature)| snapshot.get(*path) != Some(*signature))
                        .map(|(path, (_, _, is_dir))| (path.clone(), *is_dir))
                        .collect();
                    snapshot = current;

                    match target {
                        WatchTarget::Job(job_id) => {
                            let _ = tx.send(NormalizedWatchEvent::JobTriggered { job_id });
                        }
                        WatchTarget::Blueprint(blueprint_id) => {
                            let mut sent_folder_event = false;
                            let mut other_change = false;
                            if auto_correct {
                                for (path, is_dir) in &changed {
                                    if *is_dir && path.parent() == Some(root.as_path()) {
                                        let folder_name = path
                                            .file_name()
                                            .and_then(|name| name.to_str())
                                            .unwrap_or("")
                                            .to_string();
                                        let _ = tx.send(NormalizedWatchEvent::FolderCreated {
                                            blueprint_id,
                                            path: path.display().to_string(),
                                            folder_name,
                                        });
                                        sent_folder_event = true;
                                    } else {
                                        other_change = true;
                                    }
                                }
                            }
                            if !sent_folder_event || other_change {
                                let _ = tx.send(NormalizedWatchEvent::BlueprintTriggered {
                                    blueprint_id,
                                });
                            }
                        }
                    }
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

type DirSnapshot = HashMap<PathBuf, (u64, u64, bool)>;

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
        .filter(|entry| entry.path() != root)
        .filter(|entry| entry.file_type().is_file() || entry.file_type().is_dir());

    for entry in walker {
        if let Some(sig) = file_signature(entry.path()) {
            snapshot.insert(entry.path().to_path_buf(), sig);
        }
    }

    snapshot
}

fn file_signature(path: &Path) -> Option<(u64, u64, bool)> {
    let meta = std::fs::symlink_metadata(path).ok()?;
    let modified = meta
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    Some((meta.len(), modified, meta.is_dir()))
}
