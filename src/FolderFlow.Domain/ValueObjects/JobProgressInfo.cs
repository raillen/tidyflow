using System;

namespace FolderFlow.Domain.ValueObjects;

public class JobProgressInfo
{
    public Guid JobId { get; set; }
    public string JobName { get; set; } = string.Empty;
    public string CurrentFile { get; set; } = string.Empty;
    public int TotalFiles { get; set; }
    public int ProcessedFiles { get; set; }
    
    public long TotalBytes { get; set; }
    public long ProcessedBytes { get; set; }
    
    public double Percentage => TotalFiles > 0 ? (double)ProcessedFiles / TotalFiles * 100 : 0;
    public double TotalPercentage => TotalBytes > 0 ? (double)ProcessedBytes / TotalBytes * 100 : Percentage;
    
    public string Status { get; set; } = string.Empty; // COPIADO, MOVIDO, SKIP, ERRO, etc.
    public string Details { get; set; } = string.Empty;
    public double CurrentFilePercentage { get; set; }
    public double TransferSpeed { get; set; } // Bytes per second
    public TimeSpan? EstimatedTimeRemaining { get; set; }
    public DateTime Timestamp { get; set; } = DateTime.Now;

    // Premium Rolling Log
    public System.Collections.Generic.List<string> RecentFilesLog { get; set; } = new();

    public void AddLog(string message)
    {
        RecentFilesLog.Insert(0, $"[{DateTime.Now:HH:mm:ss}] {message}");
        if (RecentFilesLog.Count > 10) RecentFilesLog.RemoveAt(10);
    }
}
