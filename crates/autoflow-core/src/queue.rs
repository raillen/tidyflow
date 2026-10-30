use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

use autoflow_application::execution::{run_job, CompletedCallback, JobRunDeps, ProgressCallback};
use autoflow_application::ports::JobStore;
use autoflow_domain::{ActiveExecution, DomainError, ExecutionCompleted, ExecutionProgress};
use autoflow_infrastructure::{
    audit::SqliteAuditStore, jobs::SqliteJobStore, rollback::SqliteRollbackStore,
    settings::SqliteSettingsStore,
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub enum ExecutionEvent {
    Progress(ExecutionProgress),
    Completed(ExecutionCompleted),
}

enum QueueMessage {
    Run { job_id: Uuid, execution_id: Uuid },
    Cancel { execution_id: Uuid },
}

#[derive(Clone)]
pub struct JobQueue {
    sender: mpsc::UnboundedSender<QueueMessage>,
    active: Arc<RwLock<HashMap<Uuid, ActiveExecution>>>,
    running_jobs: Arc<RwLock<HashMap<Uuid, Uuid>>>,
    queued: Arc<RwLock<HashMap<Uuid, Uuid>>>,
    #[allow(dead_code)]
    cancelled: Arc<RwLock<HashSet<Uuid>>>,
    paused: Arc<std::sync::atomic::AtomicBool>,
    pending: Arc<Mutex<Vec<QueueMessage>>>,
    resume_notify: Arc<tokio::sync::Notify>,
    #[allow(dead_code)]
    settings: Arc<SqliteSettingsStore>,
    #[allow(dead_code)]
    rollback: Arc<SqliteRollbackStore>,
}

impl JobQueue {
    pub fn start(
        data_dir: PathBuf,
        jobs: Arc<SqliteJobStore>,
        audit: Arc<SqliteAuditStore>,
        settings: Arc<SqliteSettingsStore>,
        rollback: Arc<SqliteRollbackStore>,
        events: Arc<dyn Fn(ExecutionEvent) + Send + Sync>,
    ) -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel();
        let active = Arc::new(RwLock::new(HashMap::new()));
        let running_jobs = Arc::new(RwLock::new(HashMap::new()));
        let queued = Arc::new(RwLock::new(HashMap::new()));
        let cancelled = Arc::new(RwLock::new(HashSet::new()));
        let paused = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let pending = Arc::new(Mutex::new(Vec::new()));
        let resume_notify = Arc::new(tokio::sync::Notify::new());
        let cancel_tokens: Arc<RwLock<HashMap<Uuid, CancellationToken>>> =
            Arc::new(RwLock::new(HashMap::new()));

        let active_worker = active.clone();
        let running_worker = running_jobs.clone();
        let queued_worker = queued.clone();
        let cancelled_worker = cancelled.clone();
        let paused_worker = paused.clone();
        let pending_worker = pending.clone();
        let resume_notify_worker = resume_notify.clone();
        let cancel_worker = cancel_tokens.clone();
        let events_worker = events.clone();
        let settings_worker = settings.clone();
        let rollback_service = rollback.clone();

        tokio::spawn(async move {
            loop {
                if paused_worker.load(std::sync::atomic::Ordering::SeqCst) {
                    tokio::select! {
                        msg = receiver.recv() => {
                            match msg {
                                Some(QueueMessage::Run { job_id, execution_id }) => {
                                    pending_worker.lock().unwrap().push(QueueMessage::Run { job_id, execution_id });
                                }
                                Some(QueueMessage::Cancel { execution_id }) => {
                                    let token = cancel_worker.write().unwrap().remove(&execution_id);
                                    if let Some(token) = token {
                                        token.cancel();
                                    } else {
                                        let job_id = queued_worker.write().unwrap().remove(&execution_id);
                                        if let Some(job_id) = job_id {
                                            cancelled_worker.write().unwrap().insert(execution_id);
                                            events_worker(ExecutionEvent::Completed(ExecutionCompleted {
                                                execution_id,
                                                job_id,
                                                success: false,
                                                processed: 0,
                                                failed: 0,
                                                error_message: Some("cancelled".to_string()),
                                            }));
                                        }
                                    }
                                }
                                None => break,
                            }
                        }
                        _ = resume_notify_worker.notified() => {}
                    }
                    continue;
                }

                let pending_message = pending_worker.lock().unwrap().pop();
                let message = if let Some(message) = pending_message {
                    Some(message)
                } else {
                    receiver.recv().await
                };
                let Some(message) = message else {
                    break;
                };

                match message {
                    QueueMessage::Run {
                        job_id,
                        execution_id,
                    } => {
                        let settings_worker = settings_worker.clone();
                        queued_worker.write().unwrap().remove(&execution_id);

                        let cancelled_worker = cancelled_worker.clone();
                        if cancelled_worker.read().unwrap().contains(&execution_id) {
                            cancelled_worker.write().unwrap().remove(&execution_id);
                            events_worker(ExecutionEvent::Completed(ExecutionCompleted {
                                execution_id,
                                job_id,
                                success: false,
                                processed: 0,
                                failed: 0,
                                error_message: Some("cancelled".to_string()),
                            }));
                            continue;
                        }

                        {
                            let mut running = running_worker.write().unwrap();
                            if running.contains_key(&job_id) {
                                continue;
                            }
                            running.insert(job_id, execution_id);
                        }

                        let token = CancellationToken::new();
                        cancel_worker
                            .write()
                            .unwrap()
                            .insert(execution_id, token.clone());

                        if cancelled_worker.read().unwrap().contains(&execution_id) {
                            token.cancel();
                        }

                        let jobs = jobs.clone();
                        let audit = audit.clone();
                        let rollback = rollback.clone();
                        let events = events_worker.clone();
                        let active = active_worker.clone();
                        let running = running_worker.clone();
                        let cancel_map = cancel_worker.clone();
                        let data_dir = data_dir.clone();

                        tokio::spawn(async move {
                            let progress_cb: ProgressCallback = {
                                let active = active.clone();
                                let events = events.clone();
                                Box::new(move |progress| {
                                    {
                                        let mut map = active.write().unwrap();
                                        map.insert(
                                            progress.execution_id,
                                            ActiveExecution {
                                                execution_id: progress.execution_id,
                                                job_id: progress.job_id,
                                                job_name: progress.job_name.clone(),
                                                current_file: progress.current_file.clone(),
                                                percent: progress.percent,
                                                bytes_per_sec: progress.bytes_per_sec,
                                                recent_log: progress.recent_log.clone(),
                                            },
                                        );
                                    }
                                    events(ExecutionEvent::Progress(progress));
                                })
                            };

                            let completed_cb: CompletedCallback = {
                                let active = active.clone();
                                let events = events.clone();
                                let running = running.clone();
                                let cancel_map = cancel_map.clone();
                                Box::new(move |completed| {
                                    {
                                        active.write().unwrap().remove(&completed.execution_id);
                                        running.write().unwrap().remove(&completed.job_id);
                                        cancel_map.write().unwrap().remove(&completed.execution_id);
                                    }
                                    events(ExecutionEvent::Completed(completed));
                                })
                            };

                            let result = run_job(
                                JobRunDeps {
                                    data_dir: &data_dir,
                                    jobs: jobs.as_ref(),
                                    audit: audit.as_ref(),
                                    rollback: Some(rollback.as_ref()),
                                },
                                job_id,
                                execution_id,
                                progress_cb,
                                completed_cb,
                                token,
                            )
                            .await;

                            let notify_job = jobs.get(job_id).await.ok();
                            let notify_detail = match &result {
                                Ok(()) => None,
                                Err(error) => Some(error.to_string()),
                            };
                            if let Some(job) = notify_job {
                                let settings = settings_worker.clone();
                                let success = result.is_ok();
                                tokio::spawn(async move {
                                    if let Ok(settings) = settings.get_async().await {
                                        autoflow_application::notify::send_global_notifications(
                                            &settings,
                                            &job,
                                            success,
                                            notify_detail.as_deref(),
                                        )
                                        .await;
                                    }
                                });
                            }

                            if let Err(error) = result {
                                {
                                    active.write().unwrap().remove(&execution_id);
                                    running.write().unwrap().remove(&job_id);
                                    cancel_map.write().unwrap().remove(&execution_id);
                                    cancelled_worker.write().unwrap().remove(&execution_id);
                                }
                                events(ExecutionEvent::Completed(ExecutionCompleted {
                                    execution_id,
                                    job_id,
                                    success: false,
                                    processed: 0,
                                    failed: 0,
                                    error_message: Some(error.to_string()),
                                }));
                            }
                        });
                    }
                    QueueMessage::Cancel { execution_id } => {
                        let token = cancel_worker.write().unwrap().remove(&execution_id);
                        if let Some(token) = token {
                            token.cancel();
                            continue;
                        }

                        let job_id = queued_worker.write().unwrap().remove(&execution_id);
                        if let Some(job_id) = job_id {
                            cancelled_worker.write().unwrap().insert(execution_id);
                            events_worker(ExecutionEvent::Completed(ExecutionCompleted {
                                execution_id,
                                job_id,
                                success: false,
                                processed: 0,
                                failed: 0,
                                error_message: Some("cancelled".to_string()),
                            }));
                        }
                    }
                }
            }
        });

        Self {
            sender,
            active,
            running_jobs,
            queued,
            cancelled,
            paused,
            pending,
            resume_notify,
            settings,
            rollback: rollback_service,
        }
    }

    pub fn pause(&self) {
        self.paused.store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn resume(&self) {
        self.paused
            .store(false, std::sync::atomic::Ordering::SeqCst);
        self.resume_notify.notify_waiters();
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn stop_all(&self) -> Result<(), DomainError> {
        let execution_ids: Vec<Uuid> = self.active.read().unwrap().keys().copied().collect();
        for execution_id in execution_ids {
            let _ = self.cancel(execution_id);
        }
        self.pending.lock().unwrap().clear();
        Ok(())
    }

    pub fn enqueue(&self, job_id: Uuid) -> Result<Uuid, DomainError> {
        let mut queued = self.queued.write().unwrap();
        if self.running_jobs.read().unwrap().contains_key(&job_id)
            || queued.values().any(|queued_job| *queued_job == job_id)
        {
            return Err(DomainError::AlreadyRunning);
        }

        let execution_id = Uuid::new_v4();
        queued.insert(execution_id, job_id);
        drop(queued);

        self.sender
            .send(QueueMessage::Run {
                job_id,
                execution_id,
            })
            .map_err(|e| DomainError::Database(e.to_string()))?;
        Ok(execution_id)
    }

    pub fn cancel(&self, execution_id: Uuid) -> Result<(), DomainError> {
        if !self.active.read().unwrap().contains_key(&execution_id)
            && !self.queued.read().unwrap().contains_key(&execution_id)
        {
            return Err(DomainError::ExecutionNotFound);
        }
        self.sender
            .send(QueueMessage::Cancel { execution_id })
            .map_err(|e| DomainError::Database(e.to_string()))
    }

    pub fn list_active(&self) -> Vec<ActiveExecution> {
        self.active.read().unwrap().values().cloned().collect()
    }
}
