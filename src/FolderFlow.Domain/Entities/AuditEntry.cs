using System;

namespace FolderFlow.Domain.Entities;

public class AuditEntry
{
    public DateTime Timestamp { get; set; } = DateTime.Now;
    public string JobName { get; set; } = string.Empty;
    public string SourcePath { get; set; } = string.Empty;
    public string TargetPath { get; set; } = string.Empty;
    public string Status { get; set; } = string.Empty;
    public string Details { get; set; } = string.Empty;
}
