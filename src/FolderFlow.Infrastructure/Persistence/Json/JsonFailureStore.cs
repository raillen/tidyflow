using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using System.Threading.Tasks;
using FolderFlow.Application.Interfaces;

namespace FolderFlow.Infrastructure.Persistence.Json;

public class JsonFailureStore : IFailureStore
{
    private readonly string _retriesFolder;

    public JsonFailureStore(string? basePath = null)
    {
        var dataFolder = basePath ?? Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Data");
        _retriesFolder = Path.Combine(dataFolder, "Retries");

        if (!Directory.Exists(_retriesFolder))
        {
            Directory.CreateDirectory(_retriesFolder);
        }
    }

    public async Task SaveFailuresAsync(Guid jobId, IEnumerable<string> failedPaths)
    {
        var filePath = GetFilePath(jobId);
        var json = JsonSerializer.Serialize(failedPaths, new JsonSerializerOptions { WriteIndented = true });
        await File.WriteAllTextAsync(filePath, json);
    }

    public async Task<IEnumerable<string>> GetFailuresAsync(Guid jobId)
    {
        var filePath = GetFilePath(jobId);
        if (!File.Exists(filePath)) return Array.Empty<string>();

        try
        {
            var json = await File.ReadAllTextAsync(filePath);
            return JsonSerializer.Deserialize<List<string>>(json) ?? new List<string>();
        }
        catch
        {
            return Array.Empty<string>();
        }
    }

    public Task ClearFailuresAsync(Guid jobId)
    {
        var filePath = GetFilePath(jobId);
        if (File.Exists(filePath))
        {
            File.Delete(filePath);
        }
        return Task.CompletedTask;
    }

    private string GetFilePath(Guid jobId) => Path.Combine(_retriesFolder, $"{jobId}.json");
}
