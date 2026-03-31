using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.Threading;
using FolderFlow.Application.Interfaces;
using FolderFlow.Domain.Entities;
using FolderFlow.Domain.Enums;

namespace FolderFlow.Infrastructure.Watching;

public class NativeWatchService : IWatchService
{
    private class WatchContext : IDisposable
    {
        public FileSystemWatcher? Watcher { get; set; }
        public Timer? Timer { get; set; }
        public Job Job { get; set; } = null!;
        public Action<Job> OnFileChanged { get; set; } = null!;
        public object LockObj { get; } = new object();
        public bool IsPending { get; set; }

        public void Dispose()
        {
            Watcher?.Dispose();
            Timer?.Dispose();
        }
    }

    private readonly ConcurrentDictionary<Guid, WatchContext> _contexts = new();

    public void StartWatching(Job job, Action<Job> onFileChanged)
    {
        if (_contexts.ContainsKey(job.Id)) return;
        if (!Directory.Exists(job.SourcePath)) return;

        var context = new WatchContext
        {
            Job = job,
            OnFileChanged = onFileChanged
        };

        if (job.MonitoringMode == MonitoringMode.RealTime)
        {
            context.Timer = new Timer(OnRealTimeTimerElapsed, context, Timeout.Infinite, Timeout.Infinite);
            
            var watcher = new FileSystemWatcher(job.SourcePath)
            {
                NotifyFilter = NotifyFilters.FileName | NotifyFilters.LastWrite | NotifyFilters.DirectoryName,
                IncludeSubdirectories = true,
                EnableRaisingEvents = true
            };

            FileSystemEventHandler handler = (s, e) =>
            {
                lock (context.LockObj)
                {
                    context.IsPending = true;
                    // Reset the timer to wait for SettleTimeSeconds
                    context.Timer.Change(TimeSpan.FromSeconds(context.Job.SettleTimeSeconds), Timeout.InfiniteTimeSpan);
                }
            };

            watcher.Created += handler;
            watcher.Changed += handler;
            watcher.Renamed += (s, e) => handler(s, e);
            watcher.Deleted += handler;

            context.Watcher = watcher;
        }
        else if (job.MonitoringMode == MonitoringMode.Polling)
        {
            var interval = TimeSpan.FromSeconds(job.ScanIntervalSeconds);
            context.Timer = new Timer(OnPollingTimerElapsed, context, interval, interval);
        }

        _contexts[job.Id] = context;
    }

    private void OnRealTimeTimerElapsed(object? state)
    {
        if (state is WatchContext context)
        {
            bool shouldFire = false;
            lock (context.LockObj)
            {
                if (context.IsPending)
                {
                    context.IsPending = false;
                    shouldFire = true;
                }
            }
            if (shouldFire)
            {
                context.OnFileChanged(context.Job);
            }
        }
    }

    private void OnPollingTimerElapsed(object? state)
    {
        if (state is WatchContext context)
        {
            context.OnFileChanged(context.Job);
        }
    }

    public void StopWatching(Job job)
    {
        if (_contexts.TryRemove(job.Id, out var context))
        {
            context.Dispose();
        }
    }

    public bool IsWatching(Job job) => _contexts.ContainsKey(job.Id);

    public void Dispose()
    {
        foreach (var context in _contexts.Values)
        {
            context.Dispose();
        }
        _contexts.Clear();
    }
}
