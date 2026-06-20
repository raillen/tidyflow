import { z } from "zod";

export const auditStatusSchema = z.enum(["COPIED", "MOVED", "IGNORED", "FAILED"]);

export const auditEntrySchema = z.object({
  id: z.number().int(),
  jobId: z.string().uuid().nullable(),
  jobName: z.string(),
  sourcePath: z.string(),
  targetPath: z.string(),
  status: auditStatusSchema,
  fileSize: z.number().int(),
  durationMs: z.number(),
  details: z.string().nullable(),
  createdAt: z.string().datetime(),
});

export type AuditStatus = z.infer<typeof auditStatusSchema>;
export type AuditEntry = z.infer<typeof auditEntrySchema>;

export const AUDIT_STATUS_OPTIONS = [
  { value: "all" as const, label: "Todos" },
  { value: "COPIED" as const, label: "Copiados" },
  { value: "MOVED" as const, label: "Movidos" },
  { value: "IGNORED" as const, label: "Ignorados" },
  { value: "FAILED" as const, label: "Falhas" },
];

export function auditStatusLabel(status: AuditStatus): string {
  switch (status) {
    case "COPIED":
      return "Copiado";
    case "MOVED":
      return "Movido";
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
