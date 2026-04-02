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
    private readonly BlueprintAppService _blueprintService;

    public WatchAppService(
        IWatchService watchService, 
        IJobQueue jobQueue, 
        JobAppService jobAppService, 
        BlueprintAppService blueprintService)
    {
        _watchService = watchService;
        _jobQueue = jobQueue;
        _jobAppService = jobAppService;
        _blueprintService = blueprintService;
    }

    public async Task InitializeAsync()
    {
        _ = Task.Run(async () => {
            // Inicializa Jobs de Transferncia
            var jobs = await _jobAppService.GetAllJobsAsync();
            foreach (var job in jobs)
            {
                if (job.WatchEnabled)
                {
                    _watchService.StartWatching(job, async (j) => await _jobQueue.EnqueueAsync(j));
                }
            }

            // Inicializa Blueprints de Organizao
            var blueprints = await _blueprintService.GetAllBlueprintsAsync();
            foreach (var blueprint in blueprints)
            {
                if (blueprint.IsActive)
                {
                    _watchService.StartWatchingBlueprint(blueprint, async (b, path) => await _blueprintService.ApplyBlueprintAsync(b, path));
                }
            }
        });

        await Task.CompletedTask;
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

    public void UpdateBlueprintWatching(Blueprint blueprint)
    {
        if (blueprint.IsActive)
        {
            _watchService.StartWatchingBlueprint(blueprint, async (b, path) => await _blueprintService.ApplyBlueprintAsync(b, path));
        }
        else
        {
            _watchService.StopWatchingBlueprint(blueprint);
        }
    }
}
