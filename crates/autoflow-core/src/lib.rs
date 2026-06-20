mod queue;
mod scheduler;

pub use queue::{ExecutionEvent, JobQueue};
pub use scheduler::Scheduler;

use std::path::PathBuf;
use std::sync::Arc;

use autoflow_domain::{
    AppSettings, DomainError, HealthStatus, Job, JobSummary, SimulationReport,
};
use autoflow_infrastructure::{
    audit::SqliteAuditStore,
    database,
    jobs::SqliteJobStore,
    settings::SqliteSettingsStore,
    ui_state::{MissedScheduleEntry, SqliteMissedScheduleStore, SqliteUiStateStore},
};
use serde_json::Value;
use sqlx::SqlitePool;
use uuid::Uuid;

use autoflow_application::{jobs, ports::AuditStore};

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    data_dir: PathBuf,
    pool: SqlitePool,
    settings: Arc<SqliteSettingsStore>,
    jobs: Arc<SqliteJobStore>,
    audit: Arc<SqliteAuditStore>,
    ui_state: Arc<SqliteUiStateStore>,
    missed: Arc<SqliteMissedScheduleStore>,
    queue: JobQueue,
}

impl AppState {
    pub async fn new(
        data_dir: PathBuf,
        events: Arc<dyn Fn(ExecutionEvent) + Send + Sync>,
    ) -> Result<Self, DomainError> {
        std::fs::create_dir_all(data_dir.join("scripts"))
            .map_err(|e| DomainError::Database(e.to_string()))?;

        let pool = database::init_pool(&data_dir).await?;
        let settings = Arc::new(SqliteSettingsStore::new(pool.clone()));
        settings.ensure_default().await?;

        let jobs = Arc::new(SqliteJobStore::new(pool.clone()));
        let audit = Arc::new(SqliteAuditStore::new(pool.clone()));
        let ui_state = Arc::new(SqliteUiStateStore::new(pool.clone()));
        let missed = Arc::new(SqliteMissedScheduleStore::new(pool.clone()));
        let queue = JobQueue::start(data_dir.clone(), jobs.clone(), audit.clone(), events);
        Scheduler::start(jobs.clone(), missed.clone(), queue.clone());

        Ok(Self {
            inner: Arc::new(AppStateInner {
                data_dir,
                pool,
                settings,
                jobs,
                audit,
                ui_state,
                missed,
                queue,
            }),
        })
    }

    pub fn health(&self) -> HealthStatus {
        HealthStatus::ok(env!("CARGO_PKG_VERSION"))
    }

    pub async fn get_settings(&self) -> Result<AppSettings, DomainError> {
        self.inner.settings.get_async().await
    }

    pub async fn update_settings(&self, settings: AppSettings) -> Result<AppSettings, DomainError> {
        self.inner.settings.update_async(settings).await?;
        self.inner.settings.get_async().await
    }

    pub async fn list_jobs(&self) -> Result<Vec<JobSummary>, DomainError> {
        jobs::list_jobs(self.inner.jobs.as_ref()).await
    }

    pub async fn get_job(&self, id: Uuid) -> Result<Job, DomainError> {
        jobs::get_job(self.inner.jobs.as_ref(), id).await
    }

    pub async fn create_job(&self, job: Job) -> Result<Job, DomainError> {
        jobs::create_job(self.inner.jobs.as_ref(), job).await
    }

    pub async fn update_job(&self, job: Job) -> Result<Job, DomainError> {
        jobs::update_job(self.inner.jobs.as_ref(), job).await
    }

    pub async fn delete_job(&self, id: Uuid) -> Result<(), DomainError> {
        jobs::delete_job(self.inner.jobs.as_ref(), id).await
    }

    pub async fn simulate_job(&self, id: Uuid) -> Result<SimulationReport, DomainError> {
        jobs::simulate_job(self.inner.jobs.as_ref(), id).await
    }

    pub fn simulate_job_draft(&self, job: Job) -> Result<SimulationReport, DomainError> {
        jobs::simulate_job_draft(job)
    }

    pub fn run_job(&self, job_id: Uuid) -> Result<Uuid, DomainError> {
        self.inner.queue.enqueue(job_id)
    }

    pub fn cancel_execution(&self, execution_id: Uuid) -> Result<(), DomainError> {
        self.inner.queue.cancel(execution_id)
    }

    pub fn list_active_executions(&self) -> Vec<autoflow_domain::ActiveExecution> {
        self.inner.queue.list_active()
    }

    pub async fn list_recent_audit(
        &self,
        limit: i64,
    ) -> Result<Vec<autoflow_domain::AuditEntry>, DomainError> {
        self.inner.audit.list_recent(limit).await
    }

    pub async fn ui_state_get(&self, key: String) -> Result<Option<Value>, DomainError> {
        self.inner.ui_state.get(&key).await
    }

    pub async fn ui_state_save(&self, key: String, payload: Value) -> Result<Value, DomainError> {
        self.inner.ui_state.save(&key, payload).await
    }

    pub async fn list_missed_schedules(&self) -> Result<Vec<MissedScheduleEntry>, DomainError> {
        self.inner.missed.list_unacknowledged().await
    }

    pub async fn clear_missed_schedules(&self) -> Result<(), DomainError> {
        self.inner.missed.clear().await
    }
}
