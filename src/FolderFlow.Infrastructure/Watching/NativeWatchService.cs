using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using FolderFlow.Application.Interfaces;
using FolderFlow.Domain.Entities;

namespace FolderFlow.Infrastructure.Watching;

public class NativeWatchService : IWatchService
{
    private readonly Dictionary<Guid, FileSystemWatcher> _watchers = new();
    private readonly Dictionary<Guid, DateTime> _lastEventTimes = new();
    private readonly TimeSpan _debounceTime = TimeSpan.FromSeconds(2);

    public void StartWatching(Job job, Action<Job> onFileChanged)
    {
        if (_watchers.ContainsKey(job.Id)) return;
        if (!Directory.Exists(job.SourcePath)) return;

        var watcher = new FileSystemWatcher(job.SourcePath)
        {
            NotifyFilter = NotifyFilters.FileName | NotifyFilters.LastWrite | NotifyFilters.DirectoryName,
            IncludeSubdirectories = true,
            EnableRaisingEvents = true
        };

        FileSystemEventHandler handler = (s, e) =>
        {
            if (IsDebounced(job.Id)) return;
            onFileChanged(job);
        };

        watcher.Created += handler;
        watcher.Changed += handler;
        watcher.Renamed += (s, e) => handler(s, e);

        _watchers[job.Id] = watcher;
    }

    public void StopWatching(Job job)
    {
        if (_watchers.TryGetValue(job.Id, out var watcher))
        {
            watcher.EnableRaisingEvents = false;
            watcher.Dispose();
            _watchers.Remove(job.Id);
            _lastEventTimes.Remove(job.Id);
        }
    }

    public bool IsWatching(Job job) => _watchers.ContainsKey(job.Id);

    private bool IsDebounced(Guid jobId)
    {
        var now = DateTime.UtcNow;
        if (_lastEventTimes.TryGetValue(jobId, out var lastTime))
        {
            if (now - lastTime < _debounceTime) return true;
        }
        _lastEventTimes[jobId] = now;
        return false;
    }

    public void Dispose()
    {
        foreach (var watcher in _watchers.Values)
        {
            watcher.Dispose();
        }
        _watchers.Clear();
    }
}
