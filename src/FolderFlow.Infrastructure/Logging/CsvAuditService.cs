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
        
        if (!Directory.Exists(_reportsFolder))
        {
            Directory.CreateDirectory(_reportsFolder);
        }
    }

    public async Task SaveReportAsync(string jobName, IEnumerable<AuditEntry> entries)
    {
        var safeJobName = string.Join("_", jobName.Split(Path.GetInvalidFileNameChars()));
        var fileName = $"REPORT_{safeJobName}_{DateTime.Now:yyyyMMdd_HHmmss}.csv";
        var filePath = Path.Combine(_reportsFolder, fileName);

        var sb = new StringBuilder();
        // Header
        sb.AppendLine("Timestamp;JobName;Status;SourcePath;TargetPath;Details");

        foreach (var entry in entries)
        {
            sb.AppendLine($"{entry.Timestamp:yyyy-MM-dd HH:mm:ss};\"{entry.JobName}\";\"{entry.Status}\";\"{entry.SourcePath}\";\"{entry.TargetPath}\";\"{entry.Details}\"");
        }

        await File.WriteAllTextAsync(filePath, sb.ToString(), Encoding.UTF8);
    }
}
