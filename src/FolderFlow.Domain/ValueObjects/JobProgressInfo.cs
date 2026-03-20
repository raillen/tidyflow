using System;

namespace FolderFlow.Domain.ValueObjects;

public class JobProgressInfo
{
    public Guid JobId { get; set; }
    public string JobName { get; set; } = string.Empty;
    public string CurrentFile { get; set; } = string.Empty;
    public int TotalFiles { get; set; }
    public int ProcessedFiles { get; set; }
    
    public double Percentage => TotalFiles > 0 ? (double)ProcessedFiles / TotalFiles * 100 : 0;
}
