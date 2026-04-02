using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Threading.Tasks;
using FolderFlow.Application.Interfaces;
using FolderFlow.Domain.Entities;

namespace FolderFlow.Infrastructure.Persistence;

public class JsonBlueprintStore : IBlueprintStore
{
    private readonly string _filePath;
    private readonly List<Blueprint> _cache = new();

    public JsonBlueprintStore()
    {
        var dataFolder = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Data");
        if (!Directory.Exists(dataFolder)) Directory.CreateDirectory(dataFolder);
        _filePath = Path.Combine(dataFolder, "blueprints.json");
        _ = LoadCacheAsync();
    }

    private async Task LoadCacheAsync()
    {
        if (!File.Exists(_filePath)) return;
        try
        {
            var json = await File.ReadAllTextAsync(_filePath);
            var items = JsonSerializer.Deserialize<List<Blueprint>>(json);
            if (items != null)
            {
                _cache.Clear();
                _cache.AddRange(items);
            }
        }
        catch { }
    }

    public Task<IEnumerable<Blueprint>> GetAllAsync() => Task.FromResult(_cache.AsEnumerable());

    public Task<Blueprint?> GetByIdAsync(Guid id) => Task.FromResult(_cache.FirstOrDefault(b => b.Id == id));

    public async Task SaveAsync(Blueprint blueprint)
    {
        var existing = _cache.FirstOrDefault(b => b.Id == blueprint.Id);
        if (existing != null) _cache.Remove(existing);
        _cache.Add(blueprint);
        await SaveToFileAsync();
    }

    public async Task DeleteAsync(Guid id)
    {
        var existing = _cache.FirstOrDefault(b => b.Id == id);
        if (existing != null)
        {
            _cache.Remove(existing);
            await SaveToFileAsync();
        }
    }

    private async Task SaveToFileAsync()
    {
        try
        {
            var json = JsonSerializer.Serialize(_cache, new JsonSerializerOptions { WriteIndented = true });
            await File.WriteAllTextAsync(_filePath, json);
        }
        catch { }
    }
}
