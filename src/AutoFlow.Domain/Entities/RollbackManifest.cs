using System;
using System.Collections.Generic;

namespace AutoFlow.Domain.Entities;

public class RollbackManifest
{
    public Guid JobId { get; set; }
    public DateTime Timestamp { get; set; } = DateTime.UtcNow;
    public string JobName { get; set; } = string.Empty;
    public List<RollbackItem> Items { get; set; } = new();
}

public class RollbackItem
{
    public string SourcePath { get; set; } = string.Empty;
    public string TargetPath { get; set; } = string.Empty;
    public string Action { get; set; } = string.Empty; // "MOVIDO", "COPIADO"
}
