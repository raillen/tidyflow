import { invoke } from "@tauri-apps/api/core";
import {
  appSettingsSchema,
  healthStatusSchema,
  type AppSettings,
  type HealthStatus,
} from "$lib/contracts/settings";
import {
  activeExecutionSchema,
  executionCompletedSchema,
  executionProgressSchema,
  jobSchema,
  jobSummarySchema,
  simulationReportSchema,
  type Job,
  type JobSummary,
  type SimulationReport,
} from "$lib/contracts/job";
import { z } from "zod";
import { auditEntrySchema, type AuditEntry } from "$lib/contracts/audit";

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

export async function listRecentAudit(limit = 100): Promise<AuditEntry[]> {
  const raw: unknown = await invoke("audit_list_recent", { limit });
  return z.array(auditEntrySchema).parse(raw);
}

export { executionProgressSchema, executionCompletedSchema };
export type { AuditEntry };
