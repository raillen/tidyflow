using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.Threading.Tasks;
using FolderFlow.Application.Interfaces;
using FolderFlow.Domain.Entities;
using FolderFlow.Domain.Enums;

namespace FolderFlow.Infrastructure.Watching;

public class NativeWatchService : IWatchService
{
    private readonly ConcurrentDictionary<Guid, FileSystemWatcher> _contexts = new();
    private readonly ConcurrentDictionary<Guid, Action<Job>> _jobCallbacks = new();
    private readonly ConcurrentDictionary<Guid, Action<Blueprint, string>> _blueprintCallbacks = new();
    private readonly ConcurrentDictionary<Guid, DateTime> _lastEventTime = new();

    public void StartWatching(Job job, Action<Job> onFileChanged)
    {
        if (_contexts.ContainsKey(job.Id)) StopWatching(job);
        if (!Directory.Exists(job.SourcePath)) return;

        var watcher = CreateWatcher(job.SourcePath, job.Recursive);
        watcher.Created += (s, e) => Debounce(job.Id, job.SettleTimeSeconds, () => _jobCallbacks[job.Id](job));
        watcher.Changed += (s, e) => Debounce(job.Id, job.SettleTimeSeconds, () => _jobCallbacks[job.Id](job));
        watcher.Renamed += (s, e) => Debounce(job.Id, job.SettleTimeSeconds, () => _jobCallbacks[job.Id](job));

        _contexts[job.Id] = watcher;
        _jobCallbacks[job.Id] = onFileChanged;
    }

    public void StartWatchingBlueprint(Blueprint blueprint, Action<Blueprint, string> onChanged)
    {
        if (_contexts.ContainsKey(blueprint.Id)) StopWatchingBlueprint(blueprint);
        if (!Directory.Exists(blueprint.Path)) return;

        var watcher = CreateWatcher(blueprint.Path, false);
        // Captura o caminho especfico do evento para o Blueprint
        watcher.Created += (s, e) => Debounce(blueprint.Id, 2, () => _blueprintCallbacks[blueprint.Id](blueprint, e.FullPath));
        watcher.Renamed += (s, e) => Debounce(blueprint.Id, 2, () => _blueprintCallbacks[blueprint.Id](blueprint, e.FullPath));
        watcher.Changed += (s, e) => Debounce(blueprint.Id, 2, () => _blueprintCallbacks[blueprint.Id](blueprint, e.FullPath));

        _contexts[blueprint.Id] = watcher;
        _blueprintCallbacks[blueprint.Id] = onChanged;
    }

    private FileSystemWatcher CreateWatcher(string path, bool recursive)
    {
        return new FileSystemWatcher(path)
        {
            NotifyFilter = NotifyFilters.FileName | NotifyFilters.LastWrite | NotifyFilters.Size | NotifyFilters.DirectoryName,
            IncludeSubdirectories = recursive,
            EnableRaisingEvents = true
        };
    }

    public void StopWatching(Job job) => RemoveWatcher(job.Id);
    public void StopWatchingBlueprint(Blueprint blueprint) => RemoveWatcher(blueprint.Id);

    private void RemoveWatcher(Guid id)
    {
        if (_contexts.TryRemove(id, out var watcher))
        {
            watcher.EnableRaisingEvents = false;
            watcher.Dispose();
        }
        _jobCallbacks.TryRemove(id, out _);
        _blueprintCallbacks.TryRemove(id, out _);
    }

    private void Debounce(Guid id, int settleSeconds, Action callback)
    {
        var now = DateTime.Now;
        if (_lastEventTime.TryGetValue(id, out var lastTime))
        {
            if ((now - lastTime).TotalSeconds < settleSeconds) return;
        }

        _lastEventTime[id] = now;
        
        Task.Run(async () => {
            await Task.Delay(settleSeconds * 1000);
            callback();
        });
    }

    public bool IsWatching(Job job) => _contexts.ContainsKey(job.Id);
    public bool IsWatchingBlueprint(Blueprint blueprint) => _contexts.ContainsKey(blueprint.Id);

    public void Dispose()
    {
        foreach (var context in _contexts.Values) context.Dispose();
        _contexts.Clear();
    }
}
