mod queue;
mod scheduler;
mod watch;

pub use queue::{ExecutionEvent, JobQueue};
pub use scheduler::Scheduler;
pub use watch::{WatchService, WatchTarget};

use std::path::PathBuf;
use std::sync::Arc;

use autoflow_domain::{
    AppSettings, Blueprint, BlueprintSimulationReport, BlueprintSummary, DomainError, HealthStatus,
    Job, JobSummary, SimulationReport, TemplatePipeline, TemplatePreview, FolderPlan,
    FolderPlanPreview,
};
use autoflow_infrastructure::{
    audit::SqliteAuditStore,
    blueprints::SqliteBlueprintStore,
    database,
    jobs::SqliteJobStore,
    settings::SqliteSettingsStore,
    ui_state::{MissedScheduleEntry, SqliteMissedScheduleStore, SqliteUiStateStore},
};
use serde_json::Value;
use uuid::Uuid;

use autoflow_application::{blueprints, jobs, ports::AuditStore};

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    settings: Arc<SqliteSettingsStore>,
    jobs: Arc<SqliteJobStore>,
    blueprints: Arc<SqliteBlueprintStore>,
    audit: Arc<SqliteAuditStore>,
    ui_state: Arc<SqliteUiStateStore>,
    missed: Arc<SqliteMissedScheduleStore>,
    queue: JobQueue,
    watch: WatchService,
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
        let blueprints = Arc::new(SqliteBlueprintStore::new(pool.clone()));
        let audit = Arc::new(SqliteAuditStore::new(pool.clone()));
        let ui_state = Arc::new(SqliteUiStateStore::new(pool.clone()));
        let missed = Arc::new(SqliteMissedScheduleStore::new(pool.clone()));
        let queue = JobQueue::start(data_dir.clone(), jobs.clone(), audit.clone(), events);
        Scheduler::start(jobs.clone(), missed.clone(), queue.clone());
        let watch = WatchService::start(
            jobs.clone(),
            blueprints.clone(),
            audit.clone(),
            queue.clone(),
        );

        Ok(Self {
            inner: Arc::new(AppStateInner {
                settings,
                jobs,
                blueprints,
                audit,
                ui_state,
                missed,
                queue,
                watch,
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
        let job = jobs::create_job(self.inner.jobs.as_ref(), job).await?;
        self.inner.watch.sync_job(&job).await?;
        Ok(job)
    }

    pub async fn update_job(&self, job: Job) -> Result<Job, DomainError> {
        let job = jobs::update_job(self.inner.jobs.as_ref(), job).await?;
        self.inner.watch.sync_job(&job).await?;
        Ok(job)
    }

    pub async fn delete_job(&self, id: Uuid) -> Result<(), DomainError> {
        jobs::delete_job(self.inner.jobs.as_ref(), id).await?;
        self.inner.watch.unregister_job(id);
        Ok(())
    }

    pub async fn simulate_job(&self, id: Uuid) -> Result<SimulationReport, DomainError> {
        jobs::simulate_job(self.inner.jobs.as_ref(), id).await
    }

    pub fn simulate_job_draft(&self, job: Job) -> Result<SimulationReport, DomainError> {
        jobs::simulate_job_draft(job)
    }

    pub async fn list_blueprints(&self) -> Result<Vec<BlueprintSummary>, DomainError> {
        blueprints::list_blueprints(self.inner.blueprints.as_ref()).await
    }

    pub async fn get_blueprint(&self, id: Uuid) -> Result<Blueprint, DomainError> {
        blueprints::get_blueprint(self.inner.blueprints.as_ref(), id).await
    }

    pub async fn create_blueprint(&self, blueprint: Blueprint) -> Result<Blueprint, DomainError> {
        let blueprint = blueprints::create_blueprint(self.inner.blueprints.as_ref(), blueprint).await?;
        self.inner.watch.sync_blueprint(&blueprint).await?;
        Ok(blueprint)
    }

    pub async fn update_blueprint(&self, blueprint: Blueprint) -> Result<Blueprint, DomainError> {
        let blueprint = blueprints::update_blueprint(self.inner.blueprints.as_ref(), blueprint).await?;
        self.inner.watch.sync_blueprint(&blueprint).await?;
        Ok(blueprint)
    }

    pub async fn delete_blueprint(&self, id: Uuid) -> Result<(), DomainError> {
        blueprints::delete_blueprint(self.inner.blueprints.as_ref(), id).await?;
        self.inner.watch.unregister_blueprint(id);
        Ok(())
    }

    pub async fn simulate_blueprint(
        &self,
        id: Uuid,
    ) -> Result<BlueprintSimulationReport, DomainError> {
        blueprints::simulate_blueprint(self.inner.blueprints.as_ref(), id).await
    }

    pub async fn apply_blueprint(&self, id: Uuid) -> Result<(u32, u32), DomainError> {
        blueprints::apply_blueprint(
            self.inner.blueprints.as_ref(),
            self.inner.audit.as_ref(),
            id,
        )
        .await
    }

    pub fn preview_template(
        &self,
        pipeline: TemplatePipeline,
        sample_path: String,
    ) -> TemplatePreview {
        blueprints::preview_template(pipeline, sample_path)
    }

    pub fn preview_folder_plan(
        &self,
        root_path: String,
        folder_plan: FolderPlan,
    ) -> FolderPlanPreview {
        blueprints::preview_folder_plan(root_path, folder_plan)
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
