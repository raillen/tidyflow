import { invoke } from "@tauri-apps/api/core";
import {
  adminCommandRequestSchema,
  adminCommandQueueSummarySchema,
  adminCommandResultSchema,
  adminFleetSnapshotSchema,
  adminHeartbeatDeliverySchema,
  adminHeartbeatPayloadSchema,
  adminQueuedCommandSchema,
  adminSignedHeartbeatEnvelopeSchema,
  type AdminCommandRequest,
  type AdminCommandQueueSummary,
  type AdminCommandResult,
  type AdminFleetSnapshot,
  type AdminHeartbeatDelivery,
  type AdminHeartbeatPayload,
  type AdminQueuedCommand,
  type AdminSignedHeartbeatEnvelope,
} from "$lib/contracts/admin";
import {
  appSettingsSchema,
  generatedAdminAgentSecretSchema,
  healthStatusSchema,
  type AppSettings,
  type GeneratedAdminAgentSecret,
  type HealthStatus,
} from "$lib/contracts/settings";
import {
  activeExecutionSchema,
  executionCompletedSchema,
  executionProgressSchema,
  jobSchema,
  jobSummarySchema,
  rollbackResultSchema,
  simulationReportSchema,
  type Job,
  type JobSummary,
  type RollbackResult,
  type SimulationReport,
} from "$lib/contracts/job";
import { z } from "zod";
import {
  auditEntrySchema,
  auditExportSchema,
  auditPageSchema,
  auditQuerySchema,
  type AuditEntry,
  type AuditExport,
  type AuditExportFormat,
  type AuditPage,
  type AuditQuery,
} from "$lib/contracts/audit";
import {
  blueprintApplyResultSchema,
  blueprintSchema,
  blueprintSimulationReportSchema,
  blueprintSummarySchema,
  folderPlanPreviewSchema,
  folderPlanSchema,
  templatePipelineSchema,
  templatePreviewSchema,
  type Blueprint,
  type BlueprintApplyResult,
  type BlueprintSimulationReport,
  type BlueprintSummary,
  type FolderPlan,
  type FolderPlanPreview,
  type TemplatePipeline,
  type TemplatePreview,
} from "$lib/contracts/blueprint";

export async function fetchHealth(): Promise<HealthStatus> {
  const raw: unknown = await invoke("health");
  return healthStatusSchema.parse(raw);
}

export async function fetchSettings(): Promise<AppSettings> {
  const raw: unknown = await invoke("settings_get");
  return appSettingsSchema.parse(raw);
}

export async function saveSettings(settings: AppSettings): Promise<AppSettings> {
  const raw: unknown = await invoke("settings_update", { settings });
  return appSettingsSchema.parse(raw);
}

export async function setSecurityPin(pin: string): Promise<AppSettings> {
  const raw: unknown = await invoke("security_set_pin", { pin });
  return appSettingsSchema.parse(raw);
}

export async function verifySecurityPin(pin: string): Promise<boolean> {
  const raw: unknown = await invoke("security_verify_pin", { pin });
  return z.boolean().parse(raw);
}

export async function clearSecurityPin(): Promise<AppSettings> {
  const raw: unknown = await invoke("security_clear_pin");
  return appSettingsSchema.parse(raw);
}

export async function fetchAdminFleetSnapshot(): Promise<AdminFleetSnapshot> {
  const raw: unknown = await invoke("admin_fleet_snapshot");
  return adminFleetSnapshotSchema.parse(raw);
}

export async function fetchAdminHeartbeatPayload(): Promise<AdminHeartbeatPayload> {
  const raw: unknown = await invoke("admin_heartbeat_payload");
  return adminHeartbeatPayloadSchema.parse(raw);
}

export async function fetchAdminSignedHeartbeatPayload(): Promise<AdminSignedHeartbeatEnvelope> {
  const raw: unknown = await invoke("admin_signed_heartbeat_payload");
  return adminSignedHeartbeatEnvelopeSchema.parse(raw);
}

export async function sendAdminSignedHeartbeatOnce(): Promise<AdminHeartbeatDelivery> {
  const raw: unknown = await invoke("admin_send_signed_heartbeat_once");
  return adminHeartbeatDeliverySchema.parse(raw);
}

export async function generateAdminAgentSecret(): Promise<GeneratedAdminAgentSecret> {
  const raw: unknown = await invoke("admin_agent_secret_generate");
  return generatedAdminAgentSecretSchema.parse(raw);
}

export async function setAdminAgentSecret(secret: string): Promise<AppSettings> {
  const raw: unknown = await invoke("admin_agent_secret_set", { secret });
  return appSettingsSchema.parse(raw);
}

export async function clearAdminAgentSecret(): Promise<AppSettings> {
  const raw: unknown = await invoke("admin_agent_secret_clear");
  return appSettingsSchema.parse(raw);
}

export async function dispatchAdminCommand(
  request: AdminCommandRequest,
): Promise<AdminCommandResult> {
  const raw: unknown = await invoke("admin_dispatch_command", {
    request: adminCommandRequestSchema.parse(request),
  });
  return adminCommandResultSchema.parse(raw);
}

export async function enqueueAdminCommand(
  request: AdminCommandRequest,
  source = "local-ui",
): Promise<AdminQueuedCommand> {
  const raw: unknown = await invoke("admin_enqueue_command", {
    request: adminCommandRequestSchema.parse(request),
    source,
  });
  return adminQueuedCommandSchema.parse(raw);
}

export async function listAdminCommands(limit = 50): Promise<AdminQueuedCommand[]> {
  const raw: unknown = await invoke("admin_list_commands", { limit });
  return z.array(adminQueuedCommandSchema).parse(raw);
}

export async function fetchAdminCommandQueueSummary(): Promise<AdminCommandQueueSummary> {
  const raw: unknown = await invoke("admin_command_queue_summary");
  return adminCommandQueueSummarySchema.parse(raw);
}

export async function processNextAdminCommand(): Promise<AdminQueuedCommand | null> {
  const raw: unknown = await invoke("admin_process_next_command");
  return raw === null ? null : adminQueuedCommandSchema.parse(raw);
}

export async function listJobs(): Promise<JobSummary[]> {
  const raw: unknown = await invoke("jobs_list");
  return z.array(jobSummarySchema).parse(raw);
}

export async function getJob(id: string): Promise<Job> {
  const raw: unknown = await invoke("jobs_get", { id });
  return jobSchema.parse(raw);
}

export async function createJob(job: Job): Promise<Job> {
  const raw: unknown = await invoke("jobs_create", { job });
  return jobSchema.parse(raw);
}

export async function updateJob(job: Job): Promise<Job> {
  const raw: unknown = await invoke("jobs_update", { job });
  return jobSchema.parse(raw);
}

export async function deleteJob(id: string): Promise<void> {
  await invoke("jobs_delete", { id });
}

export async function runJob(id: string): Promise<string> {
  return z.string().uuid().parse(await invoke("jobs_run", { id }));
}

export async function simulateJob(id: string): Promise<SimulationReport> {
  const raw: unknown = await invoke("jobs_simulate", { id });
  return simulationReportSchema.parse(raw);
}

export async function simulateJobDraft(job: Job): Promise<SimulationReport> {
  const raw: unknown = await invoke("jobs_simulate_draft", { job });
  return simulationReportSchema.parse(raw);
}

export async function listActiveExecutions() {
  const raw: unknown = await invoke("executions_list_active");
  return z.array(activeExecutionSchema).parse(raw);
}

export async function cancelExecution(executionId: string): Promise<void> {
  await invoke("executions_cancel", { executionId });
}

export async function rollbackJob(jobId: string): Promise<RollbackResult> {
  const raw: unknown = await invoke("rollback_execute", { jobId });
  return rollbackResultSchema.parse(raw);
}

export async function pauseQueue(): Promise<void> {
  await invoke("queue_pause");
}

export async function resumeQueue(): Promise<void> {
  await invoke("queue_resume");
}

export async function isQueuePaused(): Promise<boolean> {
  const raw: unknown = await invoke("queue_is_paused");
  return z.boolean().parse(raw);
}

export async function stopAllExecutions(): Promise<void> {
  await invoke("queue_stop_all");
}

export async function listRecentAudit(limit = 100): Promise<AuditEntry[]> {
  const raw: unknown = await invoke("audit_list_recent", { limit });
  return z.array(auditEntrySchema).parse(raw);
}

export async function queryAudit(query: AuditQuery): Promise<AuditPage> {
  const raw: unknown = await invoke("audit_query", {
    query: auditQuerySchema.parse(query),
  });
  return auditPageSchema.parse(raw);
}

export async function exportAudit(
  query: AuditQuery,
  format: AuditExportFormat,
): Promise<AuditExport> {
  const raw: unknown = await invoke("audit_export", {
    query: auditQuerySchema.parse(query),
    format,
  });
  return auditExportSchema.parse(raw);
}

export async function clearAudit(confirm = false): Promise<void> {
  await invoke("audit_clear", { confirm });
}

export async function listBlueprints(): Promise<BlueprintSummary[]> {
  const raw: unknown = await invoke("blueprints_list");
  return z.array(blueprintSummarySchema).parse(raw);
}

export async function getBlueprint(id: string): Promise<Blueprint> {
  const raw: unknown = await invoke("blueprints_get", { id });
  return blueprintSchema.parse(raw);
}

export async function createBlueprint(blueprint: Blueprint): Promise<Blueprint> {
  const raw: unknown = await invoke("blueprints_create", { blueprint });
  return blueprintSchema.parse(raw);
}

export async function updateBlueprint(blueprint: Blueprint): Promise<Blueprint> {
  const raw: unknown = await invoke("blueprints_update", { blueprint });
  return blueprintSchema.parse(raw);
}

export async function deleteBlueprint(id: string): Promise<void> {
  await invoke("blueprints_delete", { id });
}

export async function simulateBlueprint(id: string): Promise<BlueprintSimulationReport> {
  const raw: unknown = await invoke("blueprints_simulate", { id });
  return blueprintSimulationReportSchema.parse(raw);
}

export async function simulateBlueprintWithVariables(
  id: string,
  variables: Record<string, string>,
): Promise<BlueprintSimulationReport> {
  const raw: unknown = await invoke("blueprints_simulate_with_variables", { id, variables });
  return blueprintSimulationReportSchema.parse(raw);
}

export async function applyBlueprint(id: string): Promise<BlueprintApplyResult> {
  const raw: unknown = await invoke("blueprints_apply", { id });
  return blueprintApplyResultSchema.parse(raw);
}

export async function applyBlueprintWithVariables(
  id: string,
  variables: Record<string, string>,
): Promise<BlueprintApplyResult> {
  const raw: unknown = await invoke("blueprints_apply_with_variables", { id, variables });
  return blueprintApplyResultSchema.parse(raw);
}

export async function rollbackBlueprint(id: string): Promise<RollbackResult> {
  const raw: unknown = await invoke("blueprints_rollback", { id });
  return rollbackResultSchema.parse(raw);
}

export async function previewBlueprintTemplate(
  pipeline: TemplatePipeline,
  samplePath: string,
  renameTemplate: TemplatePipeline | null = null,
): Promise<TemplatePreview> {
  const raw: unknown = await invoke("blueprints_preview_template", {
    pipeline: templatePipelineSchema.parse(pipeline),
    renameTemplate: renameTemplate ? templatePipelineSchema.parse(renameTemplate) : null,
    samplePath,
  });
  return templatePreviewSchema.parse(raw);
}

export async function previewBlueprintPlan(
  rootPath: string,
  folderPlan: FolderPlan,
  variables: Record<string, string> = {},
): Promise<FolderPlanPreview> {
  const raw: unknown = await invoke("blueprints_preview_plan", {
    rootPath,
    folderPlan: folderPlanSchema.parse(folderPlan),
    variables,
  });
  return folderPlanPreviewSchema.parse(raw);
}

export { executionProgressSchema, executionCompletedSchema };
export type { AuditEntry };
