use autoflow_core::AppState;
use autoflow_domain::{
    ActiveExecution, AppSettings, AuditEntry, Blueprint, BlueprintSimulationReport,
    BlueprintSummary, FolderPlan, FolderPlanPreview, HealthStatus, Job, JobSummary,
    SimulationReport, TemplatePipeline, TemplatePreview,
};
use autoflow_infrastructure::ui_state::MissedScheduleEntry;
use serde_json::Value;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn health(state: State<'_, AppState>) -> HealthStatus {
    state.health()
}

#[tauri::command]
pub async fn settings_get(state: State<'_, AppState>) -> Result<AppSettings, String> {
    state.get_settings().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn settings_update(
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    state
        .update_settings(settings)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn jobs_list(state: State<'_, AppState>) -> Result<Vec<JobSummary>, String> {
    state.list_jobs().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn jobs_get(state: State<'_, AppState>, id: String) -> Result<Job, String> {
    let id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state.get_job(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn jobs_create(state: State<'_, AppState>, job: Job) -> Result<Job, String> {
    state.create_job(job).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn jobs_update(state: State<'_, AppState>, job: Job) -> Result<Job, String> {
    state.update_job(job).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn jobs_delete(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state.delete_job(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn jobs_run(state: State<'_, AppState>, id: String) -> Result<String, String> {
    let id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state
        .run_job(id)
        .map(|execution_id| execution_id.to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn jobs_simulate(
    state: State<'_, AppState>,
    id: String,
) -> Result<SimulationReport, String> {
    let id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state.simulate_job(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn jobs_simulate_draft(
    state: State<'_, AppState>,
    job: Job,
) -> Result<SimulationReport, String> {
    state.simulate_job_draft(job).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn executions_list_active(state: State<'_, AppState>) -> Vec<ActiveExecution> {
    state.list_active_executions()
}

#[tauri::command]
pub fn executions_cancel(state: State<'_, AppState>, execution_id: String) -> Result<(), String> {
    let execution_id = Uuid::parse_str(&execution_id).map_err(|e| e.to_string())?;
    state
        .cancel_execution(execution_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn audit_list_recent(
    state: State<'_, AppState>,
    limit: i64,
) -> Result<Vec<AuditEntry>, String> {
    state
        .list_recent_audit(limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ui_state_get(state: State<'_, AppState>, key: String) -> Result<Option<Value>, String> {
    state.ui_state_get(key).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ui_state_save(state: State<'_, AppState>, key: String, payload: Value) -> Result<Value, String> {
    state
        .ui_state_save(key, payload)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn jobs_list_missed_schedules(
    state: State<'_, AppState>,
) -> Result<Vec<MissedScheduleEntry>, String> {
    state
        .list_missed_schedules()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn jobs_clear_missed_schedules(state: State<'_, AppState>) -> Result<(), String> {
    state.clear_missed_schedules().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn blueprints_list(state: State<'_, AppState>) -> Result<Vec<BlueprintSummary>, String> {
    state.list_blueprints().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn blueprints_get(state: State<'_, AppState>, id: String) -> Result<Blueprint, String> {
    let id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state.get_blueprint(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn blueprints_create(state: State<'_, AppState>, blueprint: Blueprint) -> Result<Blueprint, String> {
    state.create_blueprint(blueprint).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn blueprints_update(state: State<'_, AppState>, blueprint: Blueprint) -> Result<Blueprint, String> {
    state.update_blueprint(blueprint).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn blueprints_delete(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state.delete_blueprint(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn blueprints_simulate(
    state: State<'_, AppState>,
    id: String,
) -> Result<BlueprintSimulationReport, String> {
    let id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state.simulate_blueprint(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn blueprints_apply(state: State<'_, AppState>, id: String) -> Result<(u32, u32), String> {
    let id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    state.apply_blueprint(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn blueprints_preview_template(
    state: State<'_, AppState>,
    pipeline: TemplatePipeline,
    sample_path: String,
) -> TemplatePreview {
    state.preview_template(pipeline, sample_path)
}

#[tauri::command]
pub fn blueprints_preview_plan(
    state: State<'_, AppState>,
    root_path: String,
    folder_plan: FolderPlan,
) -> FolderPlanPreview {
    state.preview_folder_plan(root_path, folder_plan)
}
