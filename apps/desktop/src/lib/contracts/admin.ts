import { z } from "zod";
import { activeExecutionSchema, jobModeSchema } from "./job";
import { adminAgentModeSchema } from "./settings";

export const adminInstanceStatusSchema = z.enum(["online", "warning", "offline"]);
export const adminCommandKindSchema = z.enum([
  "startJob",
  "cancelExecution",
  "pauseJob",
  "resumeJob",
  "stopJob",
  "createJob",
  "updateJob",
  "deleteJob",
  "applySettingsPolicy",
  "requestLogs",
]);
export const adminCommandSupportSchema = z.enum(["available", "planned"]);
export const adminEnvelopeKindSchema = z.enum(["enrollment", "heartbeat", "command", "secretRotation"]);

export const adminCommandCapabilitySchema = z.object({
  kind: adminCommandKindSchema,
  label: z.string(),
  support: adminCommandSupportSchema,
  scope: z.string(),
  requiresConfirmation: z.boolean(),
});

export const adminHardwareProfileSchema = z.object({
  hostName: z.string(),
  operatingSystem: z.string(),
  architecture: z.string(),
  cpuThreads: z.number().int().min(1),
  totalMemoryMb: z.number().int().nonnegative().nullable(),
  appVersion: z.string(),
});

export const adminNetworkInterfaceSchema = z.object({
  name: z.string(),
  address: z.string().nullable(),
  kind: z.string(),
});

export const adminNetworkProfileSchema = z.object({
  domain: z.string().nullable(),
  interfaces: z.array(adminNetworkInterfaceSchema),
});

export const adminManagementProfileSchema = z.object({
  enabled: z.boolean(),
  mode: adminAgentModeSchema,
  serverUrl: z.string().nullable(),
  allowRemoteCommands: z.boolean(),
  allowBatchCommands: z.boolean(),
  heartbeatIntervalSecs: z.number().int().min(10),
  inventoryIntervalSecs: z.number().int().min(60),
});

export const adminJobRuntimeStatusSchema = z.enum(["running", "scheduled", "idle", "disabled"]);

export const adminManagedJobSchema = z.object({
  id: z.string().uuid(),
  name: z.string(),
  mode: jobModeSchema,
  enabled: z.boolean(),
  sourcePath: z.string(),
  targetPath: z.string(),
  lastRun: z.string().datetime().nullable(),
  nextRun: z.string().datetime().nullable(),
  status: adminJobRuntimeStatusSchema,
});

export const adminInstanceSnapshotSchema = z.object({
  instanceId: z.string(),
  displayName: z.string(),
  status: adminInstanceStatusSchema,
  lastSeenAt: z.string().datetime(),
  hardware: adminHardwareProfileSchema,
  network: adminNetworkProfileSchema,
  management: adminManagementProfileSchema,
  jobs: z.array(adminManagedJobSchema),
  activeExecutions: z.array(activeExecutionSchema),
  capabilities: z.array(adminCommandCapabilitySchema),
});

export const adminFleetSummarySchema = z.object({
  totalInstances: z.number().int().nonnegative(),
  onlineInstances: z.number().int().nonnegative(),
  warningInstances: z.number().int().nonnegative(),
  offlineInstances: z.number().int().nonnegative(),
  totalJobs: z.number().int().nonnegative(),
  runningJobs: z.number().int().nonnegative(),
});

export const adminFleetSnapshotSchema = z.object({
  generatedAt: z.string().datetime(),
  summary: adminFleetSummarySchema,
  instances: z.array(adminInstanceSnapshotSchema),
});

export const adminEnrollmentRequestSchema = z.object({
  instance: adminInstanceSnapshotSchema,
  requestedAt: z.string().datetime(),
});

export const adminEnrollmentTokenRequestSchema = z.object({
  token: z.string().min(1),
  instance: adminInstanceSnapshotSchema,
  agentSecret: z.string().min(1),
  requestedAt: z.string().datetime(),
});

export const adminEnrollmentResponseSchema = z.object({
  accepted: z.boolean(),
  instanceId: z.string(),
  issuedAt: z.string().datetime(),
  message: z.string(),
});

export const adminHeartbeatPayloadSchema = z.object({
  instance: adminInstanceSnapshotSchema,
  generatedAt: z.string().datetime(),
  pendingCommandCount: z.number().int().nonnegative(),
});

export const adminHeartbeatDeliverySchema = z.object({
  endpoint: z.string(),
  statusCode: z.number().int().min(100).max(599).nullable(),
  accepted: z.boolean(),
  message: z.string(),
  sentAt: z.string().datetime(),
});

export const adminAgentSecretRotationRequestSchema = z.object({
  newAgentSecret: z.string().min(1),
  requestedAt: z.string().datetime(),
});

export const adminAgentSecretRotationAcceptedSchema = z.object({
  accepted: z.boolean(),
  instanceId: z.string(),
  rotatedAt: z.string().datetime(),
  message: z.string(),
});

export const adminSignedHeartbeatEnvelopeSchema = z.object({
  schemaVersion: z.literal("admin.transport.v1"),
  kind: z.literal("heartbeat"),
  instanceId: z.string(),
  issuedAt: z.string().datetime(),
  expiresAt: z.string().datetime(),
  nonce: z.string().uuid(),
  payloadHash: z.string().min(1),
  signature: z.string().startsWith("blake3:"),
  payload: adminHeartbeatPayloadSchema,
});

export const adminSignedSecretRotationEnvelopeSchema = z.object({
  schemaVersion: z.literal("admin.transport.v1"),
  kind: z.literal("secretRotation"),
  instanceId: z.string(),
  issuedAt: z.string().datetime(),
  expiresAt: z.string().datetime(),
  nonce: z.string().uuid(),
  payloadHash: z.string().min(1),
  signature: z.string().startsWith("blake3:"),
  payload: adminAgentSecretRotationRequestSchema,
});

export const adminCommandRequestSchema = z.object({
  kind: adminCommandKindSchema,
  targetInstanceIds: z.array(z.string()).default([]),
  jobIds: z.array(z.string().uuid()).default([]),
  executionIds: z.array(z.string().uuid()).default([]),
  reason: z.string().nullable().optional(),
});

export const adminCommandTargetStatusSchema = z.enum(["accepted", "skipped", "error"]);

export const adminCommandTargetResultSchema = z.object({
  targetInstanceId: z.string(),
  status: adminCommandTargetStatusSchema,
  message: z.string(),
});

export const adminCommandResultSchema = z.object({
  accepted: z.boolean(),
  command: adminCommandKindSchema,
  results: z.array(adminCommandTargetResultSchema),
});

export const adminQueuedCommandStatusSchema = z.enum([
  "pending",
  "running",
  "completed",
  "failed",
  "skipped",
]);

export const adminQueuedCommandSchema = z.object({
  id: z.string().uuid(),
  source: z.string(),
  request: adminCommandRequestSchema,
  status: adminQueuedCommandStatusSchema,
  result: adminCommandResultSchema.nullable(),
  createdAt: z.string().datetime(),
  updatedAt: z.string().datetime(),
});

export const adminCommandQueueSummarySchema = z.object({
  pending: z.number().int().nonnegative(),
  running: z.number().int().nonnegative(),
  completed: z.number().int().nonnegative(),
  failed: z.number().int().nonnegative(),
  skipped: z.number().int().nonnegative(),
});

export type AdminInstanceStatus = z.infer<typeof adminInstanceStatusSchema>;
export type AdminCommandKind = z.infer<typeof adminCommandKindSchema>;
export type AdminCommandSupport = z.infer<typeof adminCommandSupportSchema>;
export type AdminEnvelopeKind = z.infer<typeof adminEnvelopeKindSchema>;
export type AdminCommandCapability = z.infer<typeof adminCommandCapabilitySchema>;
export type AdminHardwareProfile = z.infer<typeof adminHardwareProfileSchema>;
export type AdminNetworkInterface = z.infer<typeof adminNetworkInterfaceSchema>;
export type AdminNetworkProfile = z.infer<typeof adminNetworkProfileSchema>;
export type AdminManagementProfile = z.infer<typeof adminManagementProfileSchema>;
export type AdminJobRuntimeStatus = z.infer<typeof adminJobRuntimeStatusSchema>;
export type AdminManagedJob = z.infer<typeof adminManagedJobSchema>;
export type AdminInstanceSnapshot = z.infer<typeof adminInstanceSnapshotSchema>;
export type AdminFleetSummary = z.infer<typeof adminFleetSummarySchema>;
export type AdminFleetSnapshot = z.infer<typeof adminFleetSnapshotSchema>;
export type AdminEnrollmentRequest = z.infer<typeof adminEnrollmentRequestSchema>;
export type AdminEnrollmentTokenRequest = z.infer<typeof adminEnrollmentTokenRequestSchema>;
export type AdminEnrollmentResponse = z.infer<typeof adminEnrollmentResponseSchema>;
export type AdminHeartbeatPayload = z.infer<typeof adminHeartbeatPayloadSchema>;
export type AdminHeartbeatDelivery = z.infer<typeof adminHeartbeatDeliverySchema>;
export type AdminAgentSecretRotationRequest = z.infer<typeof adminAgentSecretRotationRequestSchema>;
export type AdminAgentSecretRotationAccepted = z.infer<
  typeof adminAgentSecretRotationAcceptedSchema
>;
export type AdminSignedHeartbeatEnvelope = z.infer<typeof adminSignedHeartbeatEnvelopeSchema>;
export type AdminSignedSecretRotationEnvelope = z.infer<
  typeof adminSignedSecretRotationEnvelopeSchema
>;
export type AdminCommandRequest = z.infer<typeof adminCommandRequestSchema>;
export type AdminCommandResult = z.infer<typeof adminCommandResultSchema>;
export type AdminQueuedCommandStatus = z.infer<typeof adminQueuedCommandStatusSchema>;
export type AdminQueuedCommand = z.infer<typeof adminQueuedCommandSchema>;
export type AdminCommandQueueSummary = z.infer<typeof adminCommandQueueSummarySchema>;

export function instanceStatusLabel(status: AdminInstanceStatus): string {
  switch (status) {
    case "online":
      return "Online";
    case "warning":
      return "Atenção";
    case "offline":
      return "Offline";
  }
}

export function jobRuntimeStatusLabel(status: AdminJobRuntimeStatus): string {
  switch (status) {
    case "running":
      return "Rodando";
    case "scheduled":
      return "Agendado";
    case "idle":
      return "Ocioso";
    case "disabled":
      return "Desativado";
  }
}

export function commandKindLabel(kind: AdminCommandKind): string {
  switch (kind) {
    case "startJob":
      return "Iniciar";
    case "cancelExecution":
      return "Cancelar execução";
    case "pauseJob":
      return "Pausar";
    case "resumeJob":
      return "Continuar";
    case "stopJob":
      return "Parar";
    case "createJob":
      return "Criar fluxo";
    case "updateJob":
      return "Editar fluxo";
    case "deleteJob":
      return "Deletar fluxo";
    case "applySettingsPolicy":
      return "Aplicar política";
    case "requestLogs":
      return "Solicitar logs";
  }
}

export function queuedCommandStatusLabel(status: AdminQueuedCommandStatus): string {
  switch (status) {
    case "pending":
      return "Pendente";
    case "running":
      return "Executando";
    case "completed":
      return "Concluído";
    case "failed":
      return "Falhou";
    case "skipped":
      return "Ignorado";
  }
}

export function createAdminCommandRequest(
  kind: AdminCommandKind,
  targetInstanceId: string,
  jobId?: string,
): AdminCommandRequest {
  return adminCommandRequestSchema.parse({
    kind,
    targetInstanceIds: [targetInstanceId],
    jobIds: jobId ? [jobId] : [],
    executionIds: [],
    reason: null,
  });
}
