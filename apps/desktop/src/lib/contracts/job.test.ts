import { describe, expect, it } from "vitest";
import { createEmptyJob, jobSchema, jobSummarySchema } from "./job";

describe("jobSchema", () => {
  it("accepts payload shaped like Rust Job", () => {
    const parsed = jobSchema.parse({
      id: "550e8400-e29b-41d4-a716-446655440000",
      name: "Backup docs",
      sourcePath: "C:\\src",
      targetPath: "D:\\backup",
      mode: "copy",
      conflict: "skip",
      filters: { includeExtensions: [".pdf"], recursive: true },
      options: { verifyAfterCopy: true },
      schedule: null,
      scripts: { timeoutSecs: 60 },
      notify: { enabled: false, events: ["completed", "failed"], channels: [] },
      enabled: true,
      lastRun: null,
    });

    expect(parsed.mode).toBe("copy");
    expect(parsed.filters.includeExtensions).toEqual([".pdf"]);
    expect(parsed.filters.recursive).toBe(true);
  });

  it("rejects empty name", () => {
    expect(() =>
      jobSchema.parse({
        ...createEmptyJob(),
        name: "",
      }),
    ).toThrow();
  });
});

describe("jobSummarySchema", () => {
  it("accepts summary from jobs_list", () => {
    const parsed = jobSummarySchema.parse({
      id: "550e8400-e29b-41d4-a716-446655440000",
      name: "Backup",
      sourcePath: "C:\\src",
      targetPath: "D:\\backup",
      mode: "move",
      enabled: false,
      lastRun: "2026-06-19T12:00:00.000Z",
    });

    expect(parsed.enabled).toBe(false);
  });
});
