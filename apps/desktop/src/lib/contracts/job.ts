import { z } from "zod";

export const jobModeSchema = z.enum(["copy", "move"]);
export const conflictStrategySchema = z.enum(["skip", "overwrite", "rename"]);
export const symlinkModeSchema = z.enum(["follow", "copyLink", "skip"]);

export const EXCLUDE_PRESETS = [
  { id: "node_modules", label: "node_modules" },
  { id: "git", label: ".git" },
  { id: "temp", label: "temp/tmp" },
  { id: "system", label: "Sistema" },
  { id: "build", label: "build/dist" },
] as const;

const isoDate = z.string().datetime().nullable().optional();

export const fileFilterSchema = z.object({
  includeExtensions: z.array(z.string()).default([]),
  excludePatterns: z.array(z.string()).default([]),
  excludePresetIds: z.array(z.string()).default([]),
  nameRegex: z.string().nullable().optional(),
  pathRegex: z.string().nullable().optional(),
  minSizeBytes: z.number().int().nonnegative().nullable().optional(),
  maxSizeBytes: z.number().int().nonnegative().nullable().optional(),
  maxDepth: z.number().int().nonnegative().nullable().optional(),
  modifiedAfter: isoDate,
  modifiedBefore: isoDate,
  createdAfter: isoDate,
  createdBefore: isoDate,
  olderThanDays: z.number().int().nonnegative().nullable().optional(),
  contentContains: z.string().nullable().optional(),
  contentMaxBytes: z.number().int().positive().default(5 * 1024 * 1024),
  contentExtensions: z.array(z.string()).default([
    ".txt", ".md", ".csv", ".json", ".log", ".xml", ".yaml", ".yml",
  ]),
  recursive: z.boolean().default(true),
  includeHidden: z.boolean().default(false),
  symlinkMode: symlinkModeSchema.default("follow"),
  skipEmptyFiles: z.boolean().default(false),
});

export const transferOptionsSchema = z.object({
  smartSync: z.boolean().default(false),
  strictHashSync: z.boolean().default(false),
  verifyAfterCopy: z.boolean().default(true),
  stopOnIntegrityError: z.boolean().default(false),
  encryptOutput: z.boolean().default(false),
  encryptPassword: z.string().nullable().optional(),
  rememberEncryptPassword: z.boolean().default(false),
  removeFilesAfterPack: z.boolean().default(false),
  packFilename: z.string().nullable().optional(),
});

export const scheduleRuleSchema = z.discriminatedUnion("kind", [
  z.object({ kind: z.literal("interval"), minutes: z.number().int().min(1) }),
  z.object({ kind: z.literal("daily"), hour: z.number().int().min(0).max(23), minute: z.number().int().min(0).max(59) }),
  z.object({
    kind: z.literal("weekly"),
    days: z.array(z.number().int().min(0).max(6)),
    hour: z.number().int().min(0).max(23),
    minute: z.number().int().min(0).max(59),
  }),
]);

export const scheduleConfigSchema = z.object({
  enabled: z.boolean(),
  timezone: z.string().default("local"),
  rule: scheduleRuleSchema,
});

export const scriptsConfigSchema = z.object({
  preScript: z.string().nullable().optional(),
  postScript: z.string().nullable().optional(),
  timeoutSecs: z.number().int().min(5).max(600).default(60),
});

export const notifyEventSchema = z.enum(["started", "completed", "failed"]);

export const notifyChannelSchema = z.discriminatedUnion("type", [
  z.object({
    type: z.literal("generic"),
    url: z.string().url(),
    headers: z.record(z.string()).default({}),
  }),
  z.object({ type: z.literal("discord"), webhookUrl: z.string().url() }),
  z.object({
    type: z.literal("telegram"),
    botToken: z.string().min(1),
    chatId: z.string().min(1),
    rememberToken: z.boolean().default(false),
  }),
]);

export const notifyConfigSchema = z.object({
  enabled: z.boolean().default(false),
  events: z.array(notifyEventSchema).default(["completed", "failed"]),
  channels: z.array(notifyChannelSchema).default([]),
});

export const jobSchema = z.object({
  id: z.string().uuid(),
  name: z.string().min(1).max(120),
  sourcePath: z.string(),
  targetPath: z.string(),
  mode: jobModeSchema,
  conflict: conflictStrategySchema,
  filters: fileFilterSchema.default({}),
  options: transferOptionsSchema.default({}),
  schedule: scheduleConfigSchema.nullable().optional(),
  scripts: scriptsConfigSchema.default({}),
  notify: notifyConfigSchema.default({}),
  enabled: z.boolean(),
  lastRun: z.string().datetime().nullable(),
  nextRun: z.string().datetime().nullable().optional(),
});

export const jobSummarySchema = z.object({
  id: z.string().uuid(),
  name: z.string(),
  sourcePath: z.string(),
  targetPath: z.string(),
  mode: jobModeSchema,
  enabled: z.boolean(),
  lastRun: z.string().datetime().nullable(),
  nextRun: z.string().datetime().nullable().optional(),
  scheduleEnabled: z.boolean().optional(),
});

export const activeExecutionSchema = z.object({
  executionId: z.string().uuid(),
  jobId: z.string().uuid(),
  jobName: z.string(),
  currentFile: z.string(),
  percent: z.number(),
  bytesPerSec: z.number(),
  recentLog: z.array(z.string()),
});

export const executionProgressSchema = activeExecutionSchema;

export const executionCompletedSchema = z.object({
  executionId: z.string().uuid(),
  jobId: z.string().uuid(),
  success: z.boolean(),
  processed: z.number().int(),
  failed: z.number().int(),
  errorMessage: z.string().nullable(),
});

export const simulationSampleSchema = z.object({
  source: z.string(),
  target: z.string(),
  action: z.string(),
});

export const simulationReportSchema = z.object({
  filesMatched: z.number().int(),
  filesSkipped: z.number().int(),
  sample: z.array(simulationSampleSchema),
  warnings: z.array(z.string()),
});

export type Job = z.infer<typeof jobSchema>;
export type JobSummary = z.infer<typeof jobSummarySchema>;
export type FileFilter = z.infer<typeof fileFilterSchema>;
export type TransferOptions = z.infer<typeof transferOptionsSchema>;
export type ScheduleConfig = z.infer<typeof scheduleConfigSchema>;
export type ScheduleRule = z.infer<typeof scheduleRuleSchema>;
export type ScriptsConfig = z.infer<typeof scriptsConfigSchema>;
export type NotifyConfig = z.infer<typeof notifyConfigSchema>;
export type NotifyChannel = z.infer<typeof notifyChannelSchema>;
export type SimulationReport = z.infer<typeof simulationReportSchema>;
export type ActiveExecution = z.infer<typeof activeExecutionSchema>;
export type ExecutionCompleted = z.infer<typeof executionCompletedSchema>;

export function createEmptyJob(): Job {
  return {
    id: crypto.randomUUID(),
    name: "Novo fluxo",
    sourcePath: "",
    targetPath: "",
    mode: "copy",
    conflict: "skip",
    filters: fileFilterSchema.parse({}),
    options: transferOptionsSchema.parse({}),
    schedule: null,
    scripts: scriptsConfigSchema.parse({}),
    notify: notifyConfigSchema.parse({}),
    enabled: true,
    lastRun: null,
    nextRun: null,
  };
}

export function countActiveFilters(filters: FileFilter): number {
  let n = 0;
  if (filters.includeExtensions.length) n++;
  if (filters.excludePatterns.length || filters.excludePresetIds.length) n++;
  if (filters.nameRegex || filters.pathRegex) n++;
  if (filters.minSizeBytes != null || filters.maxSizeBytes != null) n++;
  if (filters.maxDepth != null) n++;
  if (filters.modifiedAfter || filters.modifiedBefore || filters.createdAfter || filters.createdBefore || filters.olderThanDays) n++;
  if (filters.contentContains) n++;
  if (!filters.recursive) n++;
  if (filters.includeHidden) n++;
  if (filters.symlinkMode !== "follow") n++;
  if (filters.skipEmptyFiles) n++;
  return n;
}
