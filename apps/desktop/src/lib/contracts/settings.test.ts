import { describe, expect, it } from "vitest";
import { appSettingsSchema } from "./settings";

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
});
