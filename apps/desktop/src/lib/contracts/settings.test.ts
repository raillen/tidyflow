import { describe, expect, it } from "vitest";
import { appSettingsSchema, generatedAdminAgentSecretSchema, hashPin } from "./settings";

describe("appSettingsSchema", () => {
  it("accepts default-shaped payload from Rust", () => {
    const parsed = appSettingsSchema.parse({
      theme: "system",
      accentColor: "#0064ff",
      language: "pt-BR",
      autostart: false,
      bandwidthLimitMbps: 0,
      maxParallelFiles: 1,
      logRetentionDays: 30,
    });

    expect(parsed.language).toBe("pt-BR");
    expect(parsed.accentColor).toBe("#0064ff");
    expect(parsed.performance.maxThreads).toBe(2);
    expect(parsed.maintenance.logRetentionDays).toBe(30);
    expect(parsed.security.maskSensitivePaths).toBe(true);
    expect(parsed.admin.mode).toBe("localOnly");
  });

  it("defaults accent when missing from legacy payload", () => {
    const parsed = appSettingsSchema.parse({
      theme: "dark",
      language: "pt-BR",
      autostart: false,
      bandwidthLimitMbps: 0,
      maxParallelFiles: 1,
      logRetentionDays: 30,
    });

    expect(parsed.accentColor).toBe("#0064ff");
  });

  it("rejects invalid parallelism", () => {
    expect(() =>
      appSettingsSchema.parse({
        theme: "dark",
        accentColor: "#0064ff",
        language: "pt-BR",
        autostart: false,
        bandwidthLimitMbps: 0,
        maxParallelFiles: 0,
        logRetentionDays: 30,
      }),
    ).toThrow();
  });

  it("rejects invalid accent color", () => {
    expect(() =>
      appSettingsSchema.parse({
        theme: "dark",
        accentColor: "blue",
        language: "pt-BR",
        autostart: false,
        bandwidthLimitMbps: 0,
        maxParallelFiles: 1,
        logRetentionDays: 30,
      }),
    ).toThrow();
  });

  it("accepts robust settings sections", () => {
    const parsed = appSettingsSchema.parse({
      theme: "system",
      accentColor: "#0064ff",
      language: "pt-BR",
      interfaceFont: "public-sans",
      autostart: true,
      closeToTray: true,
      startMinimized: false,
      bandwidthLimitMbps: 20,
      maxParallelFiles: 2,
      logRetentionDays: 60,
      performance: {
        maxThreads: 4,
        memoryLimitMb: 2048,
        processPriority: "normal",
        globalBandwidthLimitMbps: 20,
        queuePollIntervalMs: 500,
        pauseWhenOnBattery: true,
      },
      security: {
        pinEnabled: true,
        accessPinHash: "abc123",
        requirePinOnStartup: true,
        lockOnMinimize: true,
        lockOnTray: true,
        encryptionEnabled: true,
        masterKeyHint: "cofre",
        maskSensitivePaths: true,
      },
      notifications: {
        enabled: true,
        desktopEnabled: true,
        webhookEnabled: true,
        webhooks: [{ name: "Ops", url: "https://example.com/hook", enabled: true }],
        smtp: { enabled: true, host: "smtp.example.com", port: 587, useTls: true },
        adminPanelEnabled: false,
        notifyOnSuccess: false,
        notifyOnFailure: true,
      },
      maintenance: {
        logRetentionDays: 60,
        autoCompactDatabase: true,
        backupEnabled: true,
        backupDirectory: "D:\\Backups",
        backupIntervalHours: 24,
        backupRetentionCount: 7,
        optimizeAfterCleanup: true,
      },
      support: {},
      about: {},
      admin: {
        enabled: true,
        mode: "managedAgent",
        instanceId: "local-123",
        displayName: "Estacao 01",
        serverUrl: "https://admin.tidyflow.local",
        enrollmentTokenConfigured: true,
        allowRemoteCommands: true,
        allowBatchCommands: true,
        heartbeatIntervalSecs: 30,
        inventoryIntervalSecs: 300,
        lastRegisteredAt: "2026-06-21T10:00:00.000Z",
      },
    });

    expect(parsed.performance.maxThreads).toBe(4);
    expect(parsed.notifications.webhooks[0].name).toBe("Ops");
    expect(parsed.admin.mode).toBe("managedAgent");
  });

  it("hashes pin without returning plain text", async () => {
    const hashed = await hashPin("1234");

    expect(hashed).toHaveLength(64);
    expect(hashed).not.toBe("1234");
  });

  it("accepts generated admin agent secret payload", () => {
    const parsed = generatedAdminAgentSecretSchema.parse({
      secret: "af_550e8400e29b41d4a716446655440000_550e8400e29b41d4a716446655440001",
      settings: {
        theme: "system",
        accentColor: "#0064ff",
        language: "pt-BR",
        autostart: false,
        bandwidthLimitMbps: 0,
        maxParallelFiles: 1,
        logRetentionDays: 30,
        admin: {
          enrollmentTokenConfigured: true,
        },
      },
    });

    expect(parsed.settings.admin.enrollmentTokenConfigured).toBe(true);
  });
});
