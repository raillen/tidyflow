use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use autoflow_application::execution::{run_job, CompletedCallback, ProgressCallback};
use autoflow_domain::{ActiveExecution, DomainError, ExecutionCompleted, ExecutionProgress};
use autoflow_infrastructure::{audit::SqliteAuditStore, jobs::SqliteJobStore};
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
}

impl JobQueue {
    pub fn start(
        data_dir: PathBuf,
        jobs: Arc<SqliteJobStore>,
        audit: Arc<SqliteAuditStore>,
        events: Arc<dyn Fn(ExecutionEvent) + Send + Sync>,
    ) -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel();
        let active = Arc::new(RwLock::new(HashMap::new()));
        let running_jobs = Arc::new(RwLock::new(HashMap::new()));
        let cancel_tokens: Arc<RwLock<HashMap<Uuid, CancellationToken>>> =
            Arc::new(RwLock::new(HashMap::new()));

        let active_worker = active.clone();
        let running_worker = running_jobs.clone();
        let cancel_worker = cancel_tokens.clone();
        let events_worker = events.clone();

        tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                match message {
                    QueueMessage::Run {
                        job_id,
                        execution_id,
                    } => {
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

                        let jobs = jobs.clone();
                        let audit = audit.clone();
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
                                &data_dir,
                                jobs.as_ref(),
                                audit.as_ref(),
                                job_id,
                                execution_id,
                                progress_cb,
                                completed_cb,
                                token,
                            )
                            .await;

                            if let Err(error) = result {
                                {
                                    active.write().unwrap().remove(&execution_id);
                                    running.write().unwrap().remove(&job_id);
                                    cancel_map.write().unwrap().remove(&execution_id);
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
                        if let Some(token) = cancel_worker.write().unwrap().remove(&execution_id) {
                            token.cancel();
                        }
                    }
                }
            }
        });

        Self {
            sender,
            active,
            running_jobs,
        }
    }

    pub fn enqueue(&self, job_id: Uuid) -> Result<Uuid, DomainError> {
        if self.running_jobs.read().unwrap().contains_key(&job_id) {
            return Err(DomainError::AlreadyRunning);
        }

        let execution_id = Uuid::new_v4();
        self.sender
            .send(QueueMessage::Run {
                job_id,
                execution_id,
            })
            .map_err(|e| DomainError::Database(e.to_string()))?;
        Ok(execution_id)
    }

    pub fn cancel(&self, execution_id: Uuid) -> Result<(), DomainError> {
        if !self.active.read().unwrap().contains_key(&execution_id) {
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
