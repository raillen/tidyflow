import { z } from "zod";

export const themeModeSchema = z.enum(["system", "light", "dark"]);
export const processPrioritySchema = z.enum(["low", "normal", "high"]);
export const adminAgentModeSchema = z.enum(["localOnly", "managedAgent"]);

export const accentColorSchema = z
  .string()
  .regex(/^#[0-9A-Fa-f]{6}$/, "Cor de destaque inválida");

export const performanceSettingsSchema = z.object({
  maxThreads: z.number().int().min(1).max(128).default(2),
  memoryLimitMb: z.number().int().min(0).default(0),
  processPriority: processPrioritySchema.default("normal"),
  globalBandwidthLimitMbps: z.number().int().min(0).default(0),
  queuePollIntervalMs: z.number().int().min(100).default(500),
  pauseWhenOnBattery: z.boolean().default(false),
});

export const securitySettingsSchema = z.object({
  pinEnabled: z.boolean().default(false),
  accessPinHash: z.string().nullable().optional(),
  requirePinOnStartup: z.boolean().default(false),
  lockOnMinimize: z.boolean().default(false),
  lockOnTray: z.boolean().default(false),
  encryptionEnabled: z.boolean().default(false),
  masterKeyHint: z.string().default(""),
  maskSensitivePaths: z.boolean().default(true),
});

export const webhookSettingsSchema = z.object({
  name: z.string().min(1),
  url: z.string(),
  enabled: z.boolean().default(true),
  secretConfigured: z.boolean().default(false),
});

export const smtpSettingsSchema = z.object({
  enabled: z.boolean().default(false),
  host: z.string().default(""),
  port: z.number().int().min(1).max(65535).default(587),
  username: z.string().default(""),
  fromAddress: z.string().default(""),
  useTls: z.boolean().default(true),
  passwordConfigured: z.boolean().default(false),
});

export const notificationSettingsSchema = z.object({
  enabled: z.boolean().default(false),
  desktopEnabled: z.boolean().default(true),
  webhookEnabled: z.boolean().default(false),
  webhooks: z.array(webhookSettingsSchema).default([]),
  smtp: smtpSettingsSchema.default({}),
  adminPanelEnabled: z.boolean().default(false),
  notifyOnSuccess: z.boolean().default(false),
  notifyOnFailure: z.boolean().default(true),
});

export const maintenanceSettingsSchema = z.object({
  logRetentionDays: z.number().int().min(0).default(30),
  autoCompactDatabase: z.boolean().default(true),
  backupEnabled: z.boolean().default(false),
  backupDirectory: z.string().default(""),
  backupIntervalHours: z.number().int().min(1).default(24),
  backupRetentionCount: z.number().int().min(1).default(7),
  optimizeAfterCleanup: z.boolean().default(false),
});

export const supportSettingsSchema = z.object({
  supportEmail: z.string().default("suporte@autoflow.local"),
  pixKey: z.string().default(""),
  bankDepositInfo: z.string().default(""),
  buyMeCoffeeUrl: z.string().default(""),
  donationsEnabled: z.boolean().default(true),
});

export const aboutSettingsSchema = z.object({
  projectName: z.string().default("AutoFlow"),
  projectDescription: z
    .string()
    .default("Automação local para organizar, copiar e mover arquivos com auditoria clara."),
  creatorName: z.string().default("Raillen Santos"),
  creatorBio: z.string().default("Criador do AutoFlow e projetos Zenith."),
  websiteUrl: z.string().default(""),
  githubUrl: z.string().default("https://github.com/raillen"),
  linkedinUrl: z.string().default(""),
});

export const adminSettingsSchema = z.object({
  enabled: z.boolean().default(true),
  mode: adminAgentModeSchema.default("localOnly"),
  instanceId: z.string().nullable().optional(),
  displayName: z.string().max(120).default(""),
  serverUrl: z.string().default(""),
  enrollmentTokenConfigured: z.boolean().default(false),
  allowRemoteCommands: z.boolean().default(false),
  allowBatchCommands: z.boolean().default(false),
  heartbeatIntervalSecs: z.number().int().min(10).max(3600).default(30),
  inventoryIntervalSecs: z.number().int().min(60).max(86400).default(300),
  lastRegisteredAt: z.string().datetime().nullable().optional(),
});

export const appSettingsSchema = z.object({
  theme: themeModeSchema.default("system"),
  accentColor: accentColorSchema.default("#0064ff"),
  language: z.string().min(1).default("pt-BR"),
  interfaceFont: z.string().min(1).default("system"),
  autostart: z.boolean().default(false),
  closeToTray: z.boolean().default(true),
  startMinimized: z.boolean().default(false),
  bandwidthLimitMbps: z.number().int().min(0).default(0),
  maxParallelFiles: z.number().int().min(1).default(1),
  logRetentionDays: z.number().int().min(0).default(30),
  performance: performanceSettingsSchema.default({}),
  security: securitySettingsSchema.default({}),
  notifications: notificationSettingsSchema.default({}),
  maintenance: maintenanceSettingsSchema.default({}),
  support: supportSettingsSchema.default({}),
  about: aboutSettingsSchema.default({}),
  admin: adminSettingsSchema.default({}),
});

export const generatedAdminAgentSecretSchema = z.object({
  secret: z.string().min(32),
  settings: appSettingsSchema,
});

export type ThemeMode = z.infer<typeof themeModeSchema>;
export type ProcessPriority = z.infer<typeof processPrioritySchema>;
export type AdminAgentMode = z.infer<typeof adminAgentModeSchema>;
export type AccentColor = z.infer<typeof accentColorSchema>;
export type PerformanceSettings = z.infer<typeof performanceSettingsSchema>;
export type SecuritySettings = z.infer<typeof securitySettingsSchema>;
export type NotificationSettings = z.infer<typeof notificationSettingsSchema>;
export type WebhookSettings = z.infer<typeof webhookSettingsSchema>;
export type SmtpSettings = z.infer<typeof smtpSettingsSchema>;
export type MaintenanceSettings = z.infer<typeof maintenanceSettingsSchema>;
export type SupportSettings = z.infer<typeof supportSettingsSchema>;
export type AboutSettings = z.infer<typeof aboutSettingsSchema>;
export type AdminSettings = z.infer<typeof adminSettingsSchema>;
export type AppSettings = z.infer<typeof appSettingsSchema>;
export type GeneratedAdminAgentSecret = z.infer<typeof generatedAdminAgentSecretSchema>;

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

export const INTERFACE_FONT_OPTIONS = [
  { id: "system", label: "Sistema" },
  { id: "inter", label: "Inter" },
  { id: "roboto", label: "Roboto" },
  { id: "public-sans", label: "Public Sans" },
  { id: "jetbrains-mono", label: "JetBrains Mono" },
] as const;

export const LANGUAGE_OPTIONS = [
  { id: "pt-BR", label: "Português (BR)" },
  { id: "en-US", label: "English (US)" },
  { id: "es-ES", label: "Español" },
] as const;

export const PROCESS_PRIORITY_OPTIONS = [
  { id: "low" as const, label: "Baixa" },
  { id: "normal" as const, label: "Normal" },
  { id: "high" as const, label: "Alta" },
] as const;

export const ADMIN_AGENT_MODE_OPTIONS = [
  { id: "localOnly" as const, label: "Somente local" },
  { id: "managedAgent" as const, label: "Agent gerenciado" },
] as const;

export const CHANGELOG_ENTRIES = [
  {
    version: "v2.1.0",
    date: "2026-06-20",
    groups: [
      {
        type: "feature",
        label: "Novas features",
        items: [
          "Painel de auditoria com analytics, filtros, detalhes e exportação CSV/JSON.",
          "Configurações avançadas para interface, performance, segurança, notificações e manutenção.",
        ],
      },
      {
        type: "fix",
        label: "Correções",
        items: ["Parser de templates aceita transforms com parâmetros como take, skip e slice."],
      },
    ],
  },
  {
    version: "v2.0.0",
    date: "2026-06-19",
    groups: [
      {
        type: "feature",
        label: "Novas features",
        items: ["Base Tauri + Svelte + Rust para automações, blueprints e histórico."],
      },
    ],
  },
] as const;

export type ChangelogFilter = "all" | "feature" | "fix" | "security" | "maintenance";

export async function hashPin(pin: string): Promise<string> {
  const normalized = pin.trim();
  if (!normalized) return "";
  const bytes = new TextEncoder().encode(normalized);
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return Array.from(new Uint8Array(digest))
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}
