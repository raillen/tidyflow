using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.Threading.Tasks;
using AutoFlow.Application.Interfaces;
using AutoFlow.Domain.Entities;
using AutoFlow.Domain.Enums;

namespace AutoFlow.Infrastructure.Watching;

public class NativeWatchService : IWatchService
{
    private readonly ConcurrentDictionary<Guid, FileSystemWatcher> _contexts = new();
    private readonly ConcurrentDictionary<Guid, Action<Job>> _jobCallbacks = new();
    private readonly ConcurrentDictionary<Guid, Action<Blueprint, string>> _blueprintCallbacks = new();
    private readonly ConcurrentDictionary<Guid, System.Timers.Timer> _debounceTimers = new();

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
        if (_debounceTimers.TryRemove(id, out var timer))
        {
            timer.Stop();
            timer.Dispose();
        }
        _jobCallbacks.TryRemove(id, out _);
        _blueprintCallbacks.TryRemove(id, out _);
    }

    private void Debounce(Guid id, int settleSeconds, Action callback)
    {
        var timer = _debounceTimers.GetOrAdd(id, _ =>
        {
            var t = new System.Timers.Timer(settleSeconds * 1000);
            t.AutoReset = false;
            t.Elapsed += (s, e) => callback();
            return t;
        });

        timer.Stop();
        timer.Interval = Math.Max(100, settleSeconds * 1000);
        timer.Start();
    }

    public bool IsWatching(Job job) => _contexts.ContainsKey(job.Id);
    public bool IsWatchingBlueprint(Blueprint blueprint) => _contexts.ContainsKey(blueprint.Id);

    public void Dispose()
    {
        foreach (var timer in _debounceTimers.Values) timer.Dispose();
        foreach (var context in _contexts.Values) context.Dispose();
        _debounceTimers.Clear();
        _contexts.Clear();
    }
}
