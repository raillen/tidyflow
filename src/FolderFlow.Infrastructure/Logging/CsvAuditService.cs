using System;
using System.Collections.Generic;
using System.IO;
using System.Text;
using System.Threading.Tasks;
using FolderFlow.Application.Interfaces;
using FolderFlow.Domain.Entities;

namespace FolderFlow.Infrastructure.Logging;

public class CsvAuditService : IAuditService
{
    private readonly string _reportsFolder;

    public CsvAuditService(string? basePath = null)
    {
        var dataFolder = basePath ?? Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Data");
        _reportsFolder = Path.Combine(dataFolder, "Reports");
        if (!Directory.Exists(_reportsFolder)) Directory.CreateDirectory(_reportsFolder);
    }

    public async Task SaveReportAsync(string jobName, IEnumerable<AuditEntry> entries)
    {
        var filePath = Path.Combine(_reportsFolder, $"{jobName}_{DateTime.Now:yyyyMMdd_HHmmss}.csv");
        var sb = new StringBuilder();
        sb.AppendLine("Timestamp;JobName;Status;SourcePath;TargetPath;Details");

        foreach (var entry in entries)
        {
            sb.AppendLine($"{entry.Timestamp:yyyy-MM-dd HH:mm:ss};\"{entry.JobName}\";\"{entry.Status}\";\"{entry.SourcePath}\";\"{entry.TargetPath}\";\"{entry.Details}\"");
        }

        await File.WriteAllTextAsync(filePath, sb.ToString(), Encoding.UTF8);
    }

    public Task<int> PurgeOldLogsAsync(int days)
    {
        if (days <= 0 || !Directory.Exists(_reportsFolder)) return Task.FromResult(0);
        
        int count = 0;
        var cutoff = DateTime.Now.AddDays(-days);
        foreach (var file in Directory.GetFiles(_reportsFolder, "*.csv"))
        {
            if (File.GetCreationTime(file) < cutoff)
            {
                File.Delete(file);
                count++;
            }
        }
        return Task.FromResult(count);
    }

    public Task<string> GetDailySummaryAsync() => Task.FromResult("Resumo CSV no implementado. Use SQLite.");
}
