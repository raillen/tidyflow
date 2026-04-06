using System;
using System.IO;
using System.Text.Json;
using System.Threading.Tasks;
using AutoFlow.Application.Interfaces;
using AutoFlow.Domain.Entities;

namespace AutoFlow.Infrastructure.Persistence;

public class JsonRollbackStore : IRollbackStore
{
    private readonly string _folderPath;

    public JsonRollbackStore(string? basePath = null)
    {
        var dataFolder = basePath ?? Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Data");
        _folderPath = Path.Combine(dataFolder, "Rollbacks");
        if (!Directory.Exists(_folderPath)) Directory.CreateDirectory(_folderPath);
    }

    public async Task SaveManifestAsync(RollbackManifest manifest)
    {
        var filePath = Path.Combine(_folderPath, $"{manifest.JobId}.json");
        var json = JsonSerializer.Serialize(manifest, new JsonSerializerOptions { WriteIndented = true });
        await File.WriteAllTextAsync(filePath, json);
    }

    public async Task<RollbackManifest?> GetLatestManifestAsync(Guid jobId)
    {
        var filePath = Path.Combine(_folderPath, $"{jobId}.json");
        if (!File.Exists(filePath)) return null;

        var json = await File.ReadAllTextAsync(filePath);
        return JsonSerializer.Deserialize<RollbackManifest>(json);
    }

    public Task ClearManifestAsync(Guid jobId)
    {
        var filePath = Path.Combine(_folderPath, $"{jobId}.json");
        if (File.Exists(filePath)) File.Delete(filePath);
        return Task.CompletedTask;
    }
}
