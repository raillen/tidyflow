import { z } from "zod";

export const auditStatusSchema = z.enum([
  "COPIED",
  "MOVED",
  "IGNORED",
  "FAILED",
  "ORGANIZED",
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

export const AUDIT_STATUS_OPTIONS = [
  { value: "all" as const, label: "Todos" },
  { value: "COPIED" as const, label: "Copiados" },
  { value: "MOVED" as const, label: "Movidos" },
  { value: "ORGANIZED" as const, label: "Organizados" },
  { value: "IGNORED" as const, label: "Ignorados" },
  { value: "FAILED" as const, label: "Falhas" },
];

export function auditStatusLabel(status: AuditStatus): string {
  switch (status) {
    case "COPIED":
      return "Copiado";
    case "MOVED":
      return "Movido";
    case "ORGANIZED":
      return "Organizado";
    case "IGNORED":
      return "Ignorado";
    case "FAILED":
      return "Falha";
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
  return new Intl.DateTimeFormat("pt-BR", {
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
