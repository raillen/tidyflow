using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Threading.Tasks;
using AutoFlow.Application.Interfaces;

namespace AutoFlow.Infrastructure.Logging;

public class SystemActivityService : ISystemActivityService
{
    private readonly string _activityFile;
    private readonly ConcurrentQueue<SystemActivity> _cache = new();
    private const int MaxCacheSize = 100;

    public SystemActivityService()
    {
        var dataFolder = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Data");
        if (!Directory.Exists(dataFolder)) Directory.CreateDirectory(dataFolder);
        _activityFile = Path.Combine(dataFolder, "system_activity.json");
        
        LoadInitial();
    }

    private void LoadInitial()
    {
        try
        {
            if (File.Exists(_activityFile))
            {
                var json = File.ReadAllText(_activityFile);
                var items = JsonSerializer.Deserialize<List<SystemActivity>>(json);
                if (items != null)
                {
                    foreach (var item in items.OrderBy(i => i.Timestamp))
                    {
                        _cache.Enqueue(item);
                    }
                }
            }
        }
        catch { }
    }

    public async Task LogActivityAsync(string message, string level = "INFO")
    {
        var activity = new SystemActivity(DateTime.Now, message, level);
        _cache.Enqueue(activity);

        while (_cache.Count > MaxCacheSize)
        {
            _cache.TryDequeue(out _);
        }

        try
        {
            var json = JsonSerializer.Serialize(_cache.ToList());
            await File.WriteAllTextAsync(_activityFile, json);
        }
        catch { }
    }

    public Task<IEnumerable<SystemActivity>> GetRecentActivitiesAsync(int count = 50)
    {
        return Task.FromResult(_cache.Reverse().Take(count));
    }
}
