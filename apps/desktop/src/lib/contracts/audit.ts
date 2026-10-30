import { get } from "svelte/store";
import { z } from "zod";
import { localeStore, t } from "$lib/i18n";

export const auditStatusSchema = z.enum([
  "COPIED",
  "MOVED",
  "IGNORED",
  "FAILED",
  "ORGANIZED",
  "ROLLED_BACK",
]);

export const auditEntrySchema = z.object({
  id: z.number().int(),
  jobId: z.string().uuid().nullable(),
  blueprintId: z.string().uuid().nullable().optional(),
  jobName: z.string(),
  sourcePath: z.string(),
  targetPath: z.string(),
  status: auditStatusSchema,
  fileSize: z.number().int(),
  durationMs: z.number(),
  details: z.string().nullable(),
  createdAt: z.string().datetime(),
});

export const auditQuerySchema = z.object({
  search: z.string().nullable().optional(),
  status: auditStatusSchema.nullable().optional(),
  jobId: z.string().uuid().nullable().optional(),
  blueprintId: z.string().uuid().nullable().optional(),
  dateFrom: z.string().datetime().nullable().optional(),
  dateTo: z.string().datetime().nullable().optional(),
  limit: z.number().int().min(1).max(1000).default(100),
  offset: z.number().int().min(0).default(0),
});

export const auditSummarySchema = z.object({
  total: z.number().int(),
  copied: z.number().int(),
  moved: z.number().int(),
  ignored: z.number().int(),
  failed: z.number().int(),
  organized: z.number().int(),
  totalBytes: z.number().int(),
  averageDurationMs: z.number(),
  latestAt: z.string().datetime().nullable(),
});

export const auditPageSchema = z.object({
  entries: z.array(auditEntrySchema),
  total: z.number().int(),
  limit: z.number().int(),
  offset: z.number().int(),
  summary: auditSummarySchema,
});

export const auditExportFormatSchema = z.enum(["csv", "json"]);

export const auditExportSchema = z.object({
  fileName: z.string(),
  mimeType: z.string(),
  content: z.string(),
});

export type AuditStatus = z.infer<typeof auditStatusSchema>;
export type AuditEntry = z.infer<typeof auditEntrySchema>;
export type AuditQuery = z.infer<typeof auditQuerySchema>;
export type AuditSummary = z.infer<typeof auditSummarySchema>;
export type AuditPage = z.infer<typeof auditPageSchema>;
export type AuditExportFormat = z.infer<typeof auditExportFormatSchema>;
export type AuditExport = z.infer<typeof auditExportSchema>;

export function auditStatusOptions() {
  return [
    { value: "all" as const, label: t("audit.status.all") },
    { value: "COPIED" as const, label: t("audit.status.copied") },
    { value: "MOVED" as const, label: t("audit.status.moved") },
    { value: "ORGANIZED" as const, label: t("audit.status.organized") },
    { value: "IGNORED" as const, label: t("audit.status.ignored") },
    { value: "FAILED" as const, label: t("audit.status.failed") },
    { value: "ROLLED_BACK" as const, label: t("audit.status.rolledBack") },
  ];
}

export function auditStatusLabel(status: AuditStatus): string {
  switch (status) {
    case "COPIED":
      return t("audit.statusLabel.copied");
    case "MOVED":
      return t("audit.statusLabel.moved");
    case "ORGANIZED":
      return t("audit.statusLabel.organized");
    case "IGNORED":
      return t("audit.statusLabel.ignored");
    case "FAILED":
      return t("audit.statusLabel.failed");
    case "ROLLED_BACK":
      return t("audit.statusLabel.rolledBack");
  }
}

export function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

export function formatDuration(ms: number): string {
  if (ms < 1000) return `${Math.round(ms)} ms`;
  return `${(ms / 1000).toFixed(2)} s`;
}

export function formatDateTime(iso: string): string {
  const locale = get(localeStore) === "en-US" ? "en-US" : "pt-BR";
  return new Intl.DateTimeFormat(locale, {
    dateStyle: "short",
    timeStyle: "medium",
  }).format(new Date(iso));
}

export function auditFailureRate(summary: AuditSummary): number {
  if (summary.total === 0) return 0;
  return summary.failed / summary.total;
}

export function createDefaultAuditQuery(): AuditQuery {
  return {
    search: null,
    status: null,
    jobId: null,
    blueprintId: null,
    dateFrom: null,
    dateTo: null,
    limit: 100,
    offset: 0,
  };
}
