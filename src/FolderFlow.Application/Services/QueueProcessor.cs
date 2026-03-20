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
    private readonly ConcurrentDictionary<Guid, bool> _activeJobs = new();
    private CancellationTokenSource? _cts;

    public int ActiveCount => _activeJobs.Count;

    public QueueProcessor(IJobQueue jobQueue, ExecutionEngine executionEngine)
    {
        _jobQueue = jobQueue;
        _executionEngine = executionEngine;
    }

    public void Start(CancellationToken cancellationToken)
    {
        _cts = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
        Task.Run(async () => await ProcessQueueAsync(_cts.Token));
    }

    private async Task ProcessQueueAsync(CancellationToken cancellationToken)
    {
        try
        {
            while (!cancellationToken.IsCancellationRequested)
            {
                var job = await _jobQueue.DequeueAsync();

                // Evitar rodar o mesmo Job em paralelo
                if (_activeJobs.TryAdd(job.Id, true))
                {
                    _ = Task.Run(async () =>
                    {
                        try
                        {
                            await _executionEngine.RunJobAsync(job, cancellationToken);
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
