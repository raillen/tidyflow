using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Linq;
using AutoFlow.Domain.ValueObjects;

namespace AutoFlow.Application.Services;

public class GlobalProgressService
{
    public event Action<JobProgressInfo>? OnProgressReported;

    private readonly ConcurrentDictionary<Guid, JobProgressInfo> _activeJobs = new();

    public void ReportProgress(JobProgressInfo progress)
    {
        _activeJobs[progress.JobId] = progress;
        OnProgressReported?.Invoke(progress);
    }

    public void CompleteJob(Guid jobId)
    {
        _activeJobs.TryRemove(jobId, out _);
    }

    public double GetCurrentGlobalSpeed()
    {
        return _activeJobs.Values.Sum(p => p.TransferSpeed);
    }

    public IEnumerable<JobProgressInfo> GetActiveJobs() => _activeJobs.Values;
}
