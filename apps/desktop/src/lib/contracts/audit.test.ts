import { describe, expect, it } from "vitest";
import {
  auditEntrySchema,
  auditExportSchema,
  auditFailureRate,
  auditPageSchema,
  auditQuerySchema,
  formatDuration,
  formatFileSize,
} from "./audit";

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

describe("audit query contracts", () => {
  it("accepts paginated audit payload with analytics", () => {
    const parsed = auditPageSchema.parse({
      entries: [],
      total: 10,
      limit: 100,
      offset: 0,
      summary: {
        total: 10,
        copied: 4,
        moved: 2,
        ignored: 1,
        failed: 3,
        organized: 0,
        totalBytes: 4096,
        averageDurationMs: 25.5,
        latestAt: "2026-06-19T12:00:00.000Z",
      },
    });

    expect(auditFailureRate(parsed.summary)).toBe(0.3);
  });

  it("normalizes default query values", () => {
    const parsed = auditQuerySchema.parse({});

    expect(parsed.limit).toBe(100);
    expect(parsed.offset).toBe(0);
  });

  it("accepts export payload", () => {
    const parsed = auditExportSchema.parse({
      fileName: "audit.csv",
      mimeType: "text/csv",
      content: "id,status\n1,COPIED\n",
    });

    expect(parsed.fileName).toBe("audit.csv");
  });
});
