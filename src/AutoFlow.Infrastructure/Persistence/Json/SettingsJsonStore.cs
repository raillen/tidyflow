using System;
using System.IO;
using System.Text.Json;
using System.Threading.Tasks;
using AutoFlow.Application.Interfaces;
using AutoFlow.Domain.Entities;

namespace AutoFlow.Infrastructure.Persistence.Json;

public class SettingsJsonStore : ISettingsStore
{
    private readonly string _filePath;

    public SettingsJsonStore(string? basePath = null)
    {
        var dataFolder = basePath ?? Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Data");
        if (!Directory.Exists(dataFolder))
        {
            Directory.CreateDirectory(dataFolder);
        }
        _filePath = Path.Combine(dataFolder, "settings.json");
    }

    public AppSettings Load()
    {
        if (!File.Exists(_filePath))
        {
            return new AppSettings();
        }

        try
        {
            var json = File.ReadAllText(_filePath);
            return JsonSerializer.Deserialize<AppSettings>(json) ?? new AppSettings();
        }
        catch
        {
            return new AppSettings();
        }
    }

    public async Task<AppSettings> LoadAsync()
    {
        if (!File.Exists(_filePath))
        {
            return new AppSettings();
        }

        try
        {
            var json = await File.ReadAllTextAsync(_filePath);
            return JsonSerializer.Deserialize<AppSettings>(json) ?? new AppSettings();
        }
        catch
        {
            return new AppSettings();
        }
    }

    public async Task SaveAsync(AppSettings settings)
    {
        var json = JsonSerializer.Serialize(settings, new JsonSerializerOptions { WriteIndented = true });
        await File.WriteAllTextAsync(_filePath, json);
    }
}
