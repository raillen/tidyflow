use std::collections::hash_map::DefaultHasher;
use std::env;
use std::hash::{Hash, Hasher};
use std::net::ToSocketAddrs;
use std::path::Path;

use autoflow_domain::{
    ActiveExecution, AdminCommandCapability, AdminCommandKind, AdminCommandRequest,
    AdminCommandResult, AdminCommandSupport, AdminCommandTargetResult, AdminCommandTargetStatus,
    AdminFleetSnapshot, AdminFleetSummary, AdminHardwareProfile, AdminInstanceSnapshot,
    AdminInstanceStatus, AdminJobRuntimeStatus, AdminManagedJob, AdminManagementProfile,
    AdminNetworkInterface, AdminNetworkProfile, AdminSettings, DomainError, JobSummary,
};
use chrono::Utc;

use crate::AppState;

pub fn derive_instance_id(data_dir: &Path) -> String {
    let mut hasher = DefaultHasher::new();
    data_dir.to_string_lossy().hash(&mut hasher);
    host_name().hash(&mut hasher);
    format!("local-{:016x}", hasher.finish())
}

pub async fn fleet_snapshot(
    state: &AppState,
    instance_id: &str,
) -> Result<AdminFleetSnapshot, DomainError> {
    let settings = state.get_settings().await?;
    let admin_settings = settings.admin.clone();
    let jobs = state.list_jobs().await?;
    let active_executions = state.list_active_executions();
    let instance = AdminInstanceSnapshot {
        instance_id: instance_id.to_string(),
        display_name: display_name(&admin_settings),
        status: local_instance_status(&jobs, &active_executions),
        last_seen_at: Utc::now(),
        hardware: hardware_profile(),
        network: network_profile(),
        management: management_profile(&admin_settings),
        jobs: jobs
            .into_iter()
            .map(|job| managed_job(job, &active_executions))
            .collect(),
        active_executions,
        capabilities: command_capabilities(),
    };
    let instances = vec![instance];

    Ok(AdminFleetSnapshot {
        generated_at: Utc::now(),
        summary: AdminFleetSummary::from_instances(&instances),
        instances,
    })
}

pub async fn dispatch_command(
    state: &AppState,
    request: AdminCommandRequest,
) -> AdminCommandResult {
    let instance_id = state.instance_id().to_string();
    let targets = if request.target_instance_ids.is_empty() {
        vec![instance_id.clone()]
    } else {
        request.target_instance_ids.clone()
    };

    let mut results = Vec::new();
    for target in targets {
        if target != instance_id {
            results.push(AdminCommandTargetResult {
                target_instance_id: target,
                status: AdminCommandTargetStatus::Skipped,
                message: "Instancia remota aguardando servidor admin".into(),
            });
            continue;
        }

        match request.kind {
            AdminCommandKind::StartJob => {
                for job_id in &request.job_ids {
                    match state.run_job(*job_id) {
                        Ok(execution_id) => results.push(AdminCommandTargetResult {
                            target_instance_id: target.clone(),
                            status: AdminCommandTargetStatus::Accepted,
                            message: format!("Fluxo iniciado: {execution_id}"),
                        }),
                        Err(error) => results.push(AdminCommandTargetResult {
                            target_instance_id: target.clone(),
                            status: AdminCommandTargetStatus::Error,
                            message: error.to_string(),
                        }),
                    }
                }
                if request.job_ids.is_empty() {
                    results.push(skipped(target, "Nenhum fluxo selecionado"));
                }
            }
            AdminCommandKind::CancelExecution => {
                for execution_id in &request.execution_ids {
                    match state.cancel_execution(*execution_id) {
                        Ok(()) => results.push(AdminCommandTargetResult {
                            target_instance_id: target.clone(),
                            status: AdminCommandTargetStatus::Accepted,
                            message: format!("Execucao cancelada: {execution_id}"),
                        }),
                        Err(error) => results.push(AdminCommandTargetResult {
                            target_instance_id: target.clone(),
                            status: AdminCommandTargetStatus::Error,
                            message: error.to_string(),
                        }),
                    }
                }
                if request.execution_ids.is_empty() {
                    results.push(skipped(target, "Nenhuma execucao selecionada"));
                }
            }
            AdminCommandKind::PauseJob => {
                toggle_jobs(state, &target, &request.job_ids, false, &mut results).await;
            }
            AdminCommandKind::ResumeJob => {
                toggle_jobs(state, &target, &request.job_ids, true, &mut results).await;
            }
            AdminCommandKind::StopJob => {
                stop_jobs(state, &target, &request, &mut results);
            }
            AdminCommandKind::DeleteJob => {
                delete_jobs(state, &target, &request.job_ids, &mut results).await;
            }
            _ => results.push(AdminCommandTargetResult {
                target_instance_id: target,
                status: AdminCommandTargetStatus::Skipped,
                message: "Comando modelado, aguardando agente remoto/servidor admin".into(),
            }),
        }
    }

    let accepted = results
        .iter()
        .any(|result| result.status == AdminCommandTargetStatus::Accepted);

    AdminCommandResult {
        accepted,
        command: request.kind,
        results,
    }
}

async fn toggle_jobs(
    state: &AppState,
    target: &str,
    job_ids: &[uuid::Uuid],
    enabled: bool,
    results: &mut Vec<AdminCommandTargetResult>,
) {
    if job_ids.is_empty() {
        results.push(skipped(target.to_string(), "Nenhum fluxo selecionado"));
        return;
    }

    for job_id in job_ids {
        match state.get_job(*job_id).await {
            Ok(mut job) => {
                job.enabled = enabled;
                match state.update_job(job).await {
                    Ok(saved) => results.push(AdminCommandTargetResult {
                        target_instance_id: target.to_string(),
                        status: AdminCommandTargetStatus::Accepted,
                        message: if enabled {
                            format!("Fluxo retomado: {}", saved.name)
                        } else {
                            format!("Fluxo pausado: {}", saved.name)
                        },
                    }),
                    Err(error) => results.push(error_result(target, error.to_string())),
                }
            }
            Err(error) => results.push(error_result(target, error.to_string())),
        }
    }
}

fn stop_jobs(
    state: &AppState,
    target: &str,
    request: &AdminCommandRequest,
    results: &mut Vec<AdminCommandTargetResult>,
) {
    let active = state.list_active_executions();
    let mut execution_ids = request.execution_ids.clone();

    if execution_ids.is_empty() {
        execution_ids = active
            .iter()
            .filter(|execution| {
                request.job_ids.is_empty() || request.job_ids.contains(&execution.job_id)
            })
            .map(|execution| execution.execution_id)
            .collect();
    }

    if execution_ids.is_empty() {
        results.push(skipped(
            target.to_string(),
            "Nenhuma execucao ativa encontrada para parar",
        ));
        return;
    }

    for execution_id in execution_ids {
        match state.cancel_execution(execution_id) {
            Ok(()) => results.push(AdminCommandTargetResult {
                target_instance_id: target.to_string(),
                status: AdminCommandTargetStatus::Accepted,
                message: format!("Execucao parada: {execution_id}"),
            }),
            Err(error) => results.push(error_result(target, error.to_string())),
        }
    }
}

async fn delete_jobs(
    state: &AppState,
    target: &str,
    job_ids: &[uuid::Uuid],
    results: &mut Vec<AdminCommandTargetResult>,
) {
    if job_ids.is_empty() {
        results.push(skipped(target.to_string(), "Nenhum fluxo selecionado"));
        return;
    }

    for job_id in job_ids {
        match state.delete_job(*job_id).await {
            Ok(()) => results.push(AdminCommandTargetResult {
                target_instance_id: target.to_string(),
                status: AdminCommandTargetStatus::Accepted,
                message: format!("Fluxo deletado: {job_id}"),
            }),
            Err(error) => results.push(error_result(target, error.to_string())),
        }
    }
}

fn skipped(target_instance_id: String, message: &str) -> AdminCommandTargetResult {
    AdminCommandTargetResult {
        target_instance_id,
        status: AdminCommandTargetStatus::Skipped,
        message: message.into(),
    }
}

fn error_result(target_instance_id: &str, message: String) -> AdminCommandTargetResult {
    AdminCommandTargetResult {
        target_instance_id: target_instance_id.into(),
        status: AdminCommandTargetStatus::Error,
        message,
    }
}

fn managed_job(job: JobSummary, active_executions: &[ActiveExecution]) -> AdminManagedJob {
    let is_running = active_executions
        .iter()
        .any(|execution| execution.job_id == job.id);
    let status = if is_running {
        AdminJobRuntimeStatus::Running
    } else if !job.enabled {
        AdminJobRuntimeStatus::Disabled
    } else if job.next_run.is_some() {
        AdminJobRuntimeStatus::Scheduled
    } else {
        AdminJobRuntimeStatus::Idle
    };

    AdminManagedJob {
        id: job.id,
        name: job.name,
        mode: job.mode,
        enabled: job.enabled,
        source_path: job.source_path,
        target_path: job.target_path,
        last_run: job.last_run,
        next_run: job.next_run,
        status,
    }
}

fn local_instance_status(
    jobs: &[JobSummary],
    active_executions: &[ActiveExecution],
) -> AdminInstanceStatus {
    if !active_executions.is_empty() {
        return AdminInstanceStatus::Online;
    }
    if jobs.iter().any(|job| job.enabled) {
        AdminInstanceStatus::Online
    } else {
        AdminInstanceStatus::Warning
    }
}

fn hardware_profile() -> AdminHardwareProfile {
    AdminHardwareProfile {
        host_name: host_name(),
        operating_system: env::consts::OS.into(),
        architecture: env::consts::ARCH.into(),
        cpu_threads: std::thread::available_parallelism()
            .map(|value| value.get() as u32)
            .unwrap_or(1),
        total_memory_mb: None,
        app_version: env!("CARGO_PKG_VERSION").into(),
    }
}

fn display_name(settings: &AdminSettings) -> String {
    if settings.display_name.is_empty() {
        host_name()
    } else {
        settings.display_name.clone()
    }
}

fn management_profile(settings: &AdminSettings) -> AdminManagementProfile {
    AdminManagementProfile {
        enabled: settings.enabled,
        mode: settings.mode,
        server_url: Some(settings.server_url.clone()).filter(|value| !value.is_empty()),
        allow_remote_commands: settings.allow_remote_commands,
        allow_batch_commands: settings.allow_batch_commands,
        heartbeat_interval_secs: settings.heartbeat_interval_secs,
        inventory_interval_secs: settings.inventory_interval_secs,
    }
}

fn network_profile() -> AdminNetworkProfile {
    let host = host_name();
    let mut interfaces = vec![AdminNetworkInterface {
        name: "hostname".into(),
        address: Some(host.clone()),
        kind: "host".into(),
    }];

    if let Ok(addresses) = (host.as_str(), 0).to_socket_addrs() {
        for address in addresses.take(4) {
            interfaces.push(AdminNetworkInterface {
                name: "resolved".into(),
                address: Some(address.ip().to_string()),
                kind: "ip".into(),
            });
        }
    }

    AdminNetworkProfile {
        domain: env::var("USERDOMAIN")
            .ok()
            .filter(|value| !value.is_empty()),
        interfaces,
    }
}

fn command_capabilities() -> Vec<AdminCommandCapability> {
    vec![
        capability(
            AdminCommandKind::StartJob,
            "Iniciar fluxo",
            AdminCommandSupport::Available,
            "maquina/fluxo",
            false,
        ),
        capability(
            AdminCommandKind::CancelExecution,
            "Cancelar execucao",
            AdminCommandSupport::Available,
            "maquina/execucao",
            true,
        ),
        capability(
            AdminCommandKind::PauseJob,
            "Pausar fluxo",
            AdminCommandSupport::Available,
            "maquina/fluxo/lote",
            true,
        ),
        capability(
            AdminCommandKind::ResumeJob,
            "Continuar fluxo",
            AdminCommandSupport::Available,
            "maquina/fluxo/lote",
            false,
        ),
        capability(
            AdminCommandKind::StopJob,
            "Parar fluxo",
            AdminCommandSupport::Available,
            "maquina/fluxo/lote",
            true,
        ),
        capability(
            AdminCommandKind::CreateJob,
            "Criar fluxo remoto",
            AdminCommandSupport::Planned,
            "maquina/lote",
            true,
        ),
        capability(
            AdminCommandKind::UpdateJob,
            "Editar fluxo remoto",
            AdminCommandSupport::Planned,
            "maquina/lote",
            true,
        ),
        capability(
            AdminCommandKind::DeleteJob,
            "Deletar fluxo remoto",
            AdminCommandSupport::Available,
            "maquina/lote",
            true,
        ),
        capability(
            AdminCommandKind::ApplySettingsPolicy,
            "Aplicar politica",
            AdminCommandSupport::Planned,
            "grupo/lote",
            true,
        ),
        capability(
            AdminCommandKind::RequestLogs,
            "Solicitar logs",
            AdminCommandSupport::Planned,
            "maquina/lote",
            false,
        ),
    ]
}

fn capability(
    kind: AdminCommandKind,
    label: &str,
    support: AdminCommandSupport,
    scope: &str,
    requires_confirmation: bool,
) -> AdminCommandCapability {
    AdminCommandCapability {
        kind,
        label: label.into(),
        support,
        scope: scope.into(),
        requires_confirmation,
    }
}

fn host_name() -> String {
    env::var("COMPUTERNAME")
        .or_else(|_| env::var("HOSTNAME"))
        .unwrap_or_else(|_| "local-machine".into())
}
