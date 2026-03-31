using System;
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
    private CancellationTokenSource? _cts;

    public SimpleScheduler(IJobQueue jobQueue, JobAppService jobAppService)
    {
        _jobQueue = jobQueue;
        _jobAppService = jobAppService;
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
            catch (Exception)
            {
                // No MVP, log simples ou ignorar
            }

            await Task.Delay(TimeSpan.FromSeconds(1), cancellationToken);
        }
    }
}
