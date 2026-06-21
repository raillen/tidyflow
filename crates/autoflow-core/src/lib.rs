mod admin;
mod queue;
mod scheduler;
mod watch;

pub use queue::{ExecutionEvent, JobQueue};
pub use scheduler::Scheduler;
pub use watch::{WatchService, WatchTarget};

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use autoflow_domain::{
    AdminAgentMode, AdminCommandQueueSummary, AdminCommandRequest, AdminCommandResult,
    AdminEnvelopeKind, AdminFleetSnapshot, AdminHeartbeatDelivery, AdminHeartbeatPayload,
    AdminQueuedCommand, AdminSignedEnvelope, AppSettings, AuditExport, AuditExportFormat,
    AuditPage, AuditQuery, AuditStatus, Blueprint, BlueprintSimulationReport, BlueprintSummary,
    DomainError, FolderPlan, FolderPlanPreview, HealthStatus, Job, JobSummary, NewAuditEntry,
    SimulationReport, TemplatePipeline, TemplatePreview,
};
use autoflow_infrastructure::{
    admin::SqliteAdminCommandStore,
    audit::SqliteAuditStore,
    blueprints::SqliteBlueprintStore,
    database,
    jobs::SqliteJobStore,
    secrets::AdminSecretStore,
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
    admin_commands: Arc<SqliteAdminCommandStore>,
    admin_secrets: AdminSecretStore,
    queue: JobQueue,
    watch: WatchService,
    instance_id: String,
}

impl AppState {
    pub async fn new(
        data_dir: PathBuf,
        events: Arc<dyn Fn(ExecutionEvent) + Send + Sync>,
    ) -> Result<Self, DomainError> {
        std::fs::create_dir_all(data_dir.join("scripts"))
            .map_err(|e| DomainError::Database(e.to_string()))?;

        let pool = database::init_pool(&data_dir).await?;
        let derived_instance_id = admin::derive_instance_id(&data_dir);
        let settings = Arc::new(SqliteSettingsStore::new(pool.clone()));
        settings.ensure_default().await?;
        let mut current_settings = settings.get_async().await?.normalized();
        let instance_id = current_settings
            .admin
            .instance_id
            .clone()
            .unwrap_or_else(|| derived_instance_id.clone());
        if current_settings.admin.instance_id.as_deref() != Some(instance_id.as_str()) {
            current_settings.admin.instance_id = Some(instance_id.clone());
            settings.update_async(current_settings).await?;
        }

        let jobs = Arc::new(SqliteJobStore::new(pool.clone()));
        let blueprints = Arc::new(SqliteBlueprintStore::new(pool.clone()));
        let audit = Arc::new(SqliteAuditStore::new(pool.clone()));
        let ui_state = Arc::new(SqliteUiStateStore::new(pool.clone()));
        let missed = Arc::new(SqliteMissedScheduleStore::new(pool.clone()));
        let admin_commands = Arc::new(SqliteAdminCommandStore::new(pool.clone()));
        let admin_secrets = AdminSecretStore::new();
        let queue = JobQueue::start(data_dir.clone(), jobs.clone(), audit.clone(), events);
        Scheduler::start(jobs.clone(), missed.clone(), queue.clone());
        let watch = WatchService::start(
            jobs.clone(),
            blueprints.clone(),
            audit.clone(),
            queue.clone(),
        );

        let state = Self {
            inner: Arc::new(AppStateInner {
                settings,
                jobs,
                blueprints,
                audit,
                ui_state,
                missed,
                admin_commands,
                admin_secrets,
                queue,
                watch,
                instance_id,
            }),
        };
        state.start_admin_heartbeat_worker();

        Ok(state)
    }

    pub fn instance_id(&self) -> &str {
        &self.inner.instance_id
    }

    pub fn health(&self) -> HealthStatus {
        HealthStatus::ok(env!("CARGO_PKG_VERSION"))
    }

    pub async fn admin_fleet_snapshot(&self) -> Result<AdminFleetSnapshot, DomainError> {
        admin::fleet_snapshot(self, self.instance_id()).await
    }

    pub async fn admin_heartbeat_payload(&self) -> Result<AdminHeartbeatPayload, DomainError> {
        let snapshot = self.admin_fleet_snapshot().await?;
        let instance = snapshot.instances.into_iter().next().ok_or_else(|| {
            DomainError::Validation("admin snapshot has no local instance".into())
        })?;
        let pending_command_count = self.inner.admin_commands.summary().await?.pending;

        Ok(AdminHeartbeatPayload {
            instance,
            generated_at: chrono::Utc::now(),
            pending_command_count,
        })
    }

    pub async fn admin_signed_heartbeat_payload(
        &self,
    ) -> Result<AdminSignedEnvelope<AdminHeartbeatPayload>, DomainError> {
        let signing_secret = self
            .inner
            .admin_secrets
            .get_agent_secret()?
            .ok_or_else(|| {
                DomainError::Validation(
                    "admin agent secret is not configured in the OS keyring".into(),
                )
            })?;
        let payload = self.admin_heartbeat_payload().await?;
        let ttl_secs = (payload.instance.management.heartbeat_interval_secs * 3).clamp(30, 3600);

        AdminSignedEnvelope::sign(
            AdminEnvelopeKind::Heartbeat,
            self.instance_id().to_string(),
            payload,
            &signing_secret,
            ttl_secs,
        )
    }

    pub async fn admin_send_signed_heartbeat_once(
        &self,
    ) -> Result<AdminHeartbeatDelivery, DomainError> {
        let settings = self.get_settings().await?;
        if !settings.admin.enabled {
            return Err(DomainError::Validation(
                "admin monitoring is disabled".into(),
            ));
        }
        if settings.admin.server_url.trim().is_empty() {
            return Err(DomainError::Validation(
                "admin server URL is required before sending heartbeat".into(),
            ));
        }

        let endpoint = admin_server_endpoint(
            &settings.admin.server_url,
            &format!("/api/agents/{}/heartbeat", self.instance_id()),
        );
        let envelope = self.admin_signed_heartbeat_payload().await?;
        let sent_at = chrono::Utc::now();
        let response = reqwest::Client::new()
            .post(&endpoint)
            .json(&envelope)
            .send()
            .await
            .map_err(|error| DomainError::Database(error.to_string()))?;

        let status = response.status();
        let status_code = Some(status.as_u16());
        let body = response.text().await.unwrap_or_default();
        let message = if body.trim().is_empty() {
            status
                .canonical_reason()
                .unwrap_or("heartbeat sent")
                .to_string()
        } else {
            truncate_message(body.trim(), 500)
        };

        Ok(AdminHeartbeatDelivery {
            endpoint,
            status_code,
            accepted: status.is_success(),
            message,
            sent_at,
        })
    }

    fn start_admin_heartbeat_worker(&self) {
        let state = self.clone();

        tokio::spawn(async move {
            loop {
                let wait_secs = match state.get_settings().await {
                    Ok(settings) => {
                        let wait_secs = admin_heartbeat_wait_secs(&settings);
                        if should_send_admin_heartbeat(&settings) {
                            if let Err(error) = state.admin_send_signed_heartbeat_once().await {
                                tracing::warn!(
                                    target = "autoflow::admin",
                                    error = %error,
                                    "failed to send admin heartbeat"
                                );
                            }
                        }
                        wait_secs
                    }
                    Err(error) => {
                        tracing::warn!(
                            target = "autoflow::admin",
                            error = %error,
                            "failed to read settings for admin heartbeat"
                        );
                        60
                    }
                };

                tokio::time::sleep(Duration::from_secs(wait_secs)).await;
            }
        });
    }

    pub async fn admin_generate_agent_secret(&self) -> Result<(String, AppSettings), DomainError> {
        let secret = generate_admin_agent_secret();
        self.admin_set_agent_secret(secret.clone()).await?;
        let settings = self.get_settings().await?;
        Ok((secret, settings))
    }

    pub async fn admin_set_agent_secret(&self, secret: String) -> Result<AppSettings, DomainError> {
        self.inner.admin_secrets.set_agent_secret(&secret)?;
        self.set_admin_secret_configured(true).await
    }

    pub async fn admin_clear_agent_secret(&self) -> Result<AppSettings, DomainError> {
        self.inner.admin_secrets.clear_agent_secret()?;
        self.set_admin_secret_configured(false).await
    }

    pub async fn admin_dispatch_command(&self, request: AdminCommandRequest) -> AdminCommandResult {
        self.admin_dispatch_command_from(request, "direct").await
    }

    pub async fn admin_enqueue_command(
        &self,
        request: AdminCommandRequest,
        source: String,
    ) -> Result<AdminQueuedCommand, DomainError> {
        let queued = self.inner.admin_commands.enqueue(&source, request).await?;
        self.record_admin_audit(
            "enqueue",
            &queued.source,
            &queued.request,
            None,
            AuditStatus::Organized,
        )
        .await;
        Ok(queued)
    }

    pub async fn admin_list_commands(
        &self,
        limit: i64,
    ) -> Result<Vec<AdminQueuedCommand>, DomainError> {
        self.inner.admin_commands.list_recent(limit).await
    }

    pub async fn admin_command_queue_summary(
        &self,
    ) -> Result<AdminCommandQueueSummary, DomainError> {
        self.inner.admin_commands.summary().await
    }

    pub async fn admin_process_next_command(
        &self,
    ) -> Result<Option<AdminQueuedCommand>, DomainError> {
        let Some(command) = self.inner.admin_commands.next_pending().await? else {
            return Ok(None);
        };

        self.inner.admin_commands.mark_running(command.id).await?;
        let source = format!("queue:{}", command.id);
        let result = self
            .admin_dispatch_command_from(command.request.clone(), &source)
            .await;
        let finished = self.inner.admin_commands.finish(command.id, result).await?;
        Ok(Some(finished))
    }

    async fn admin_dispatch_command_from(
        &self,
        request: AdminCommandRequest,
        source: &str,
    ) -> AdminCommandResult {
        let result = admin::dispatch_command(self, request.clone()).await;
        let status = audit_status_for_admin_result(&result);
        self.record_admin_audit("dispatch", source, &request, Some(&result), status)
            .await;
        result
    }

    async fn record_admin_audit(
        &self,
        action: &str,
        source: &str,
        request: &AdminCommandRequest,
        result: Option<&AdminCommandResult>,
        status: AuditStatus,
    ) {
        let target_instances = if request.target_instance_ids.is_empty() {
            self.instance_id().to_string()
        } else {
            request.target_instance_ids.join(",")
        };
        let details = serde_json::json!({
            "scope": "admin",
            "action": action,
            "source": source,
            "request": request,
            "result": result,
        });

        let entry = NewAuditEntry {
            job_id: request.job_ids.first().copied(),
            blueprint_id: None,
            job_name: format!("Admin: {}", admin_command_name(request.kind)),
            source_path: format!("admin://{source}"),
            target_path: format!("admin://{target_instances}"),
            status,
            file_size: 0,
            duration_ms: 0.0,
            details: Some(details.to_string()),
        };

        if let Err(error) = self.inner.audit.append(entry).await {
            tracing::warn!(
                target = "autoflow::admin",
                error = %error,
                "failed to write admin audit entry"
            );
        }
    }

    pub async fn get_settings(&self) -> Result<AppSettings, DomainError> {
        self.inner.settings.get_async().await
    }

    pub async fn update_settings(&self, settings: AppSettings) -> Result<AppSettings, DomainError> {
        self.inner.settings.update_async(settings).await?;
        self.inner.settings.get_async().await
    }

    async fn set_admin_secret_configured(
        &self,
        configured: bool,
    ) -> Result<AppSettings, DomainError> {
        let mut settings = self.get_settings().await?;
        settings.admin.enrollment_token_configured = configured;
        self.update_settings(settings).await
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
        let blueprint =
            blueprints::create_blueprint(self.inner.blueprints.as_ref(), blueprint).await?;
        self.inner.watch.sync_blueprint(&blueprint).await?;
        Ok(blueprint)
    }

    pub async fn update_blueprint(&self, blueprint: Blueprint) -> Result<Blueprint, DomainError> {
        let blueprint =
            blueprints::update_blueprint(self.inner.blueprints.as_ref(), blueprint).await?;
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

    pub async fn query_audit(&self, query: AuditQuery) -> Result<AuditPage, DomainError> {
        self.inner.audit.query(query).await
    }

    pub async fn export_audit(
        &self,
        query: AuditQuery,
        format: AuditExportFormat,
    ) -> Result<AuditExport, DomainError> {
        let mut export_query = query.normalized();
        export_query.offset = 0;
        export_query.limit = export_query.limit.clamp(1, 10_000);
        let page = self.inner.audit.query(export_query).await?;
        let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");

        match format {
            AuditExportFormat::Json => Ok(AuditExport {
                file_name: format!("autoflow-audit-{stamp}.json"),
                mime_type: "application/json".into(),
                content: serde_json::to_string_pretty(&page.entries)
                    .map_err(|e| DomainError::Database(e.to_string()))?,
            }),
            AuditExportFormat::Csv => Ok(AuditExport {
                file_name: format!("autoflow-audit-{stamp}.csv"),
                mime_type: "text/csv;charset=utf-8".into(),
                content: audit_entries_to_csv(&page.entries),
            }),
        }
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

fn audit_entries_to_csv(entries: &[autoflow_domain::AuditEntry]) -> String {
    let mut out = String::from(
        "id,created_at,status,job_name,source_path,target_path,file_size,duration_ms,details\n",
    );
    for entry in entries {
        let row = [
            entry.id.to_string(),
            entry.created_at.to_rfc3339(),
            format!("{:?}", entry.status).to_ascii_uppercase(),
            entry.job_name.clone(),
            entry.source_path.clone(),
            entry.target_path.clone(),
            entry.file_size.to_string(),
            entry.duration_ms.to_string(),
            entry.details.clone().unwrap_or_default(),
        ];
        out.push_str(
            &row.iter()
                .map(|value| csv_escape(value))
                .collect::<Vec<_>>()
                .join(","),
        );
        out.push('\n');
    }
    out
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn audit_status_for_admin_result(result: &AdminCommandResult) -> AuditStatus {
    if result
        .results
        .iter()
        .any(|entry| entry.status == autoflow_domain::AdminCommandTargetStatus::Error)
    {
        AuditStatus::Failed
    } else if result.accepted {
        AuditStatus::Organized
    } else {
        AuditStatus::Ignored
    }
}

fn admin_command_name(kind: autoflow_domain::AdminCommandKind) -> &'static str {
    match kind {
        autoflow_domain::AdminCommandKind::StartJob => "startJob",
        autoflow_domain::AdminCommandKind::CancelExecution => "cancelExecution",
        autoflow_domain::AdminCommandKind::PauseJob => "pauseJob",
        autoflow_domain::AdminCommandKind::ResumeJob => "resumeJob",
        autoflow_domain::AdminCommandKind::StopJob => "stopJob",
        autoflow_domain::AdminCommandKind::CreateJob => "createJob",
        autoflow_domain::AdminCommandKind::UpdateJob => "updateJob",
        autoflow_domain::AdminCommandKind::DeleteJob => "deleteJob",
        autoflow_domain::AdminCommandKind::ApplySettingsPolicy => "applySettingsPolicy",
        autoflow_domain::AdminCommandKind::RequestLogs => "requestLogs",
    }
}

fn generate_admin_agent_secret() -> String {
    format!("af_{}_{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

fn admin_server_endpoint(server_url: &str, path: &str) -> String {
    format!("{}{}", server_url.trim().trim_end_matches('/'), path)
}

fn truncate_message(message: &str, max_chars: usize) -> String {
    message.chars().take(max_chars).collect()
}

fn should_send_admin_heartbeat(settings: &AppSettings) -> bool {
    settings.admin.enabled
        && settings.admin.mode == AdminAgentMode::ManagedAgent
        && settings.admin.enrollment_token_configured
        && !settings.admin.server_url.trim().is_empty()
}

fn admin_heartbeat_wait_secs(settings: &AppSettings) -> u64 {
    settings.admin.heartbeat_interval_secs.clamp(10, 3600) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_heartbeat_worker_requires_managed_agent_ready_state() {
        let mut settings = AppSettings::default();
        settings.admin.enabled = true;
        settings.admin.mode = AdminAgentMode::ManagedAgent;
        settings.admin.server_url = "https://admin.autoflow.local".into();
        settings.admin.enrollment_token_configured = true;

        assert!(should_send_admin_heartbeat(&settings));

        settings.admin.enrollment_token_configured = false;
        assert!(!should_send_admin_heartbeat(&settings));
    }

    #[test]
    fn admin_heartbeat_wait_is_clamped_to_safe_range() {
        let mut settings = AppSettings::default();

        settings.admin.heartbeat_interval_secs = 1;
        assert_eq!(admin_heartbeat_wait_secs(&settings), 10);

        settings.admin.heartbeat_interval_secs = 7_200;
        assert_eq!(admin_heartbeat_wait_secs(&settings), 3_600);
    }
}
