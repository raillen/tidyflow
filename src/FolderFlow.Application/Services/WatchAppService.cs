using System.Collections.Generic;
using System.Threading.Tasks;
using FolderFlow.Application.Interfaces;
using FolderFlow.Domain.Entities;

namespace FolderFlow.Application.Services;

public class WatchAppService
{
    private readonly IWatchService _watchService;
    private readonly IJobQueue _jobQueue;
    private readonly JobAppService _jobAppService;

    public WatchAppService(IWatchService watchService, IJobQueue jobQueue, JobAppService jobAppService)
    {
        _watchService = watchService;
        _jobQueue = jobQueue;
        _jobAppService = jobAppService;
    }

    public async Task InitializeAsync()
    {
        var jobs = await _jobAppService.GetAllJobsAsync();
        foreach (var job in jobs)
        {
            if (job.WatchEnabled)
            {
                _watchService.StartWatching(job, async (j) => await _jobQueue.EnqueueAsync(j));
            }
        }
    }

    public void UpdateJobWatching(Job job)
    {
        if (job.WatchEnabled)
        {
            _watchService.StartWatching(job, async (j) => await _jobQueue.EnqueueAsync(j));
        }
        else
        {
            _watchService.StopWatching(job);
        }
    }
}
