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
    private readonly ConcurrentDictionary<Guid, CancellationTokenSource> _activeJobs = new();
    private CancellationTokenSource? _cts;

    public int ActiveCount => _activeJobs.Count;

    public QueueProcessor(IJobQueue jobQueue, ExecutionEngine executionEngine)
    {
        _jobQueue = jobQueue;
        _executionEngine = executionEngine;
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
            while (!cancellationToken.IsCancellationRequested)
            {
                // Se estiver pausado, aguarda um pouco antes de tentar pegar o prximo job
                if (_jobQueue.IsPaused)
                {
                    await Task.Delay(1000, cancellationToken);
                    continue;
                }

                var job = await _jobQueue.DequeueAsync();

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
                        catch (Exception)
                        {
                            // Erros j so logados pelo ExecutionEngine
                        }
                        finally
                        {
                            _activeJobs.TryRemove(job.Id, out _);
                        }
                    }, cancellationToken);
                }
            }
        }
        catch (OperationCanceledException) { }
    }
}
