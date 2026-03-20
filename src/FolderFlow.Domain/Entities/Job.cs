using System;
using System.Collections.Generic;
using FolderFlow.Domain.Enums;

namespace FolderFlow.Domain.Entities;

public class Job
{
    public Guid Id { get; set; } = Guid.NewGuid();
    public string Name { get; set; } = string.Empty;
    public string SourcePath { get; set; } = string.Empty;
    public string TargetPath { get; set; } = string.Empty;
    public JobMode Mode { get; set; } = JobMode.Copy;
    public bool Recursive { get; set; } = true;
    public ConflictMode ConflictMode { get; set; } = ConflictMode.Skip;
    
    // Automação e Segurança
    public bool SmartSync { get; set; } = true;
    public bool WatchEnabled { get; set; } = false;
    public bool VerifyHash { get; set; } = false;
    public bool EnableTrash { get; set; } = true;
    public int SettleTimeSeconds { get; set; } = 5;

    // Filtros
    public List<string> IncludeExtensions { get; set; } = new();
    public List<string> ExcludePatterns { get; set; } = new();
    public string? NameRegex { get; set; }
    public long? MinSizeKB { get; set; }
    public long? MaxSizeKB { get; set; }
    public int? ModifiedWithinDays { get; set; }

    // Agendamento
    public ScheduleType ScheduleType { get; set; } = ScheduleType.None;
    public int IntervalMinutes { get; set; } = 60;
    public string? ScheduleTime { get; set; }
    public DateTime? LastRun { get; set; }
    public DateTime? NextRun { get; set; }
}
