import { describe, expect, it } from "vitest";
import { auditEntrySchema, formatFileSize, formatDuration } from "./audit";

describe("auditEntrySchema", () => {
  it("accepts payload from Rust audit_list_recent", () => {
    const parsed = auditEntrySchema.parse({
      id: 1,
      jobId: "550e8400-e29b-41d4-a716-446655440000",
      jobName: "Backup",
      sourcePath: "C:\\src\\file.pdf",
      targetPath: "D:\\backup\\file.pdf",
      status: "COPIED",
      fileSize: 2048,
      durationMs: 12.5,
      details: null,
      createdAt: "2026-06-19T12:00:00.000Z",
    });

    expect(parsed.status).toBe("COPIED");
  });
});

describe("formatters", () => {
  it("formats file sizes", () => {
    expect(formatFileSize(512)).toBe("512 B");
    expect(formatFileSize(2048)).toBe("2.0 KB");
  });

  it("formats duration", () => {
    expect(formatDuration(450)).toBe("450 ms");
    expect(formatDuration(1500)).toBe("1.50 s");
  });
});
