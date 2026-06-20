import { z } from "zod";

export const themeModeSchema = z.enum(["system", "light", "dark"]);

export const accentColorSchema = z
  .string()
  .regex(/^#[0-9A-Fa-f]{6}$/, "Cor de destaque inválida");

export const appSettingsSchema = z.object({
  theme: themeModeSchema,
  accentColor: accentColorSchema.default("#0064ff"),
  language: z.string().min(1),
  autostart: z.boolean(),
  bandwidthLimitMbps: z.number().int().min(0),
  maxParallelFiles: z.number().int().min(1),
  logRetentionDays: z.number().int().min(0),
});

export type ThemeMode = z.infer<typeof themeModeSchema>;
export type AccentColor = z.infer<typeof accentColorSchema>;
export type AppSettings = z.infer<typeof appSettingsSchema>;

export const healthStatusSchema = z.object({
  status: z.string(),
  version: z.string(),
  core: z.string(),
});

export type HealthStatus = z.infer<typeof healthStatusSchema>;

export const ACCENT_PRESETS = [
  { id: "blue", label: "Azul", value: "#0064ff" },
  { id: "violet", label: "Violeta", value: "#7c3aed" },
  { id: "teal", label: "Teal", value: "#0d9488" },
  { id: "green", label: "Verde", value: "#16a34a" },
  { id: "amber", label: "Âmbar", value: "#d97706" },
  { id: "rose", label: "Rosa", value: "#e11d48" },
] as const;

export const THEME_OPTIONS = [
  { id: "system" as const, label: "Sistema" },
  { id: "light" as const, label: "Claro" },
  { id: "dark" as const, label: "Escuro" },
] as const;
