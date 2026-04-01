using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using FolderFlow.Application.Interfaces;
using FolderFlow.Domain.Entities;
using FolderFlow.Domain.Enums;
using FolderFlow.Domain.Policies;

namespace FolderFlow.Application.Services;

public class SimpleScheduler : ISchedulerService
{
    private readonly IJobQueue _jobQueue;
    private readonly JobAppService _jobAppService;
    private readonly ISystemActivityService _activityService;
    private CancellationTokenSource? _cts;

    public SimpleScheduler(IJobQueue jobQueue, JobAppService jobAppService, ISystemActivityService activityService)
    {
        _jobQueue = jobQueue;
        _jobAppService = jobAppService;
        _activityService = activityService;
    }

    public void Start(CancellationToken cancellationToken)
    {
        _cts = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
        Task.Run(async () => await RunSchedulerLoopAsync(_cts.Token));
    }

    private async Task RunSchedulerLoopAsync(CancellationToken cancellationToken)
    {
        while (!cancellationToken.IsCancellationRequested)
        {
            try
            {
                var jobs = await _jobAppService.GetAllJobsAsync();
                var now = DateTime.Now;

                foreach (var job in jobs)
                {
                    if (SchedulePolicy.ShouldRun(job, now))
                    {
                        job.LastRun = now;
                        await _jobAppService.SaveJobAsync(job);
                        await _jobQueue.EnqueueAsync(job);
                    }
                }
            }
            catch (Exception ex)
            {
                await _activityService.LogActivityAsync($"Erro no loop do agendador: {ex.Message}", "ERROR");
                await Task.Delay(5000, cancellationToken); // Aguarda mais em caso de erro persistente
            }

            await Task.Delay(TimeSpan.FromSeconds(1), cancellationToken);
        }
    }

    public async Task<IEnumerable<UpcomingJobInfo>> GetUpcomingJobsAsync(int count = 5)
    {
        var jobs = (await _jobAppService.GetAllJobsAsync()).ToList();
        var now = DateTime.Now;

        return jobs.Select(j => new { Job = j, Next = SchedulePolicy.GetNextRun(j, now) })
                   .Where(x => x.Next.HasValue)
                   .OrderBy(x => x.Next)
                   .Take(count)
                   .Select(x => {
                       var diff = x.Next!.Value - now;
                       string remaining;
                       if (diff.TotalDays >= 1) remaining = $"em {(int)diff.TotalDays} dias";
                       else if (diff.TotalHours >= 1) remaining = $"em {(int)diff.TotalHours}h";
                       else remaining = $"em {(int)diff.TotalMinutes}min";

                       return new UpcomingJobInfo {
                           JobName = x.Job.Name,
                           NextRun = x.Next.Value,
                           TimeRemaining = remaining
                       };
                   })
                   .ToList();
    }
}
