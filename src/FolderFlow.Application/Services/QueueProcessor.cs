using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using FolderFlow.Application.Interfaces;
using FolderFlow.Domain.Entities;

namespace FolderFlow.Application.Services;

public class QueueProcessor
{
    private readonly IJobQueue _jobQueue;
    private readonly ExecutionEngine _executionEngine;
    private readonly ISettingsStore _settingsStore;
    private readonly ConcurrentDictionary<Guid, CancellationTokenSource> _activeJobs = new();
    private CancellationTokenSource? _cts;
    private SemaphoreSlim _semaphore;

    public int ActiveCount => _activeJobs.Count;

    public QueueProcessor(IJobQueue jobQueue, ExecutionEngine executionEngine, ISettingsStore settingsStore)
    {
        _jobQueue = jobQueue;
        _executionEngine = executionEngine;
        _settingsStore = settingsStore;
        _semaphore = new SemaphoreSlim(2);
    }

    public bool IsJobActive(Guid jobId) => _activeJobs.ContainsKey(jobId);

    public void Start(CancellationToken cancellationToken)
    {
        _cts = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
        Task.Run(async () => await ProcessQueueAsync(_cts.Token));
    }

    public void StopJob(Guid jobId)
    {
        if (_activeJobs.TryGetValue(jobId, out var cts))
        {
            cts.Cancel();
        }
    }

    public bool IsPaused => _jobQueue.IsPaused;

    public void TogglePause() => _jobQueue.IsPaused = !_jobQueue.IsPaused;

    public void StopAll()
    {
        foreach (var jobId in _activeJobs.Keys)
        {
            StopJob(jobId);
        }
    }

    private async Task ProcessQueueAsync(CancellationToken cancellationToken)
    {
        try
        {
            var settings = await _settingsStore.LoadAsync();
            _semaphore = new SemaphoreSlim(settings.MaxDegreeOfParallelism);

            while (!cancellationToken.IsCancellationRequested)
            {
                if (_jobQueue.IsPaused)
                {
                    await Task.Delay(1000, cancellationToken);
                    continue;
                }

                // Aguarda o semforo antes de pegar da fila
                await _semaphore.WaitAsync(cancellationToken);

                var job = await _jobQueue.DequeueAsync();
                if (job == null)
                {
                    _semaphore.Release();
                    continue;
                }

                // Evitar rodar o mesmo Job em paralelo
                var jobCts = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
                if (_activeJobs.TryAdd(job.Id, jobCts))
                {
                    _ = Task.Run(async () =>
                    {
                        try
                        {
                            await _executionEngine.RunJobAsync(job, jobCts.Token);
                        }
                        catch (Exception) { }
                        finally
                        {
                            _activeJobs.TryRemove(job.Id, out _);
                            _semaphore.Release();
                        }
                    }, cancellationToken);
                }
                else
                {
                    _semaphore.Release();
                }
            }
        }
        catch (OperationCanceledException) { }
    }
}
