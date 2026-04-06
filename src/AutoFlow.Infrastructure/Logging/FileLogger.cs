using System;
using System.IO;
using System.Threading.Tasks;
using AutoFlow.Application.Interfaces;

namespace AutoFlow.Infrastructure.Logging;

public class FileLogger : IAppLogger
{
    private readonly string _logPath;

    public FileLogger(string? basePath = null)
    {
        var dataFolder = basePath ?? Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Data");
        if (!Directory.Exists(dataFolder)) Directory.CreateDirectory(dataFolder);
        _logPath = Path.Combine(dataFolder, "app.log");
    }

    public async Task LogAsync(string message, string level = "INFO")
    {
        var logLine = $"[{DateTime.Now:yyyy-MM-dd HH:mm:ss}] [{level}] {message}{Environment.NewLine}";
        await File.AppendAllTextAsync(_logPath, logLine);
    }
}
