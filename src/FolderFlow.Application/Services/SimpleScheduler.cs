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
    private readonly IAuditService _auditService;
    private readonly ISettingsStore _settingsStore;
    private readonly IExternalNotificationService _notificationService;
    private readonly ILocalizationService _localizationService;
    private CancellationTokenSource? _cts;

    public SimpleScheduler(
        IJobQueue jobQueue, 
        JobAppService jobAppService, 
        ISystemActivityService activityService,
        IAuditService auditService,
        ISettingsStore settingsStore,
        IExternalNotificationService notificationService,
        ILocalizationService localizationService)
    {
        _jobQueue = jobQueue;
        _jobAppService = jobAppService;
        _activityService = activityService;
        _auditService = auditService;
        _settingsStore = settingsStore;
        _notificationService = notificationService;
        _localizationService = localizationService;
    }

    public void Start(CancellationToken cancellationToken)
    {
        _cts = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
        Task.Run(async () => await RunSchedulerLoopAsync(_cts.Token));
    }

    private DateTime _lastMaintenanceDate = DateTime.MinValue;

    private async Task RunSchedulerLoopAsync(CancellationToken cancellationToken)
    {
        while (!cancellationToken.IsCancellationRequested)
        {
            try
            {
                var now = DateTime.Now;

                // Tarefa de Manuteno Diria (Meia-noite)
                if (now.Date > _lastMaintenanceDate.Date)
                {
                    await RunDailyMaintenanceAsync();
                    _lastMaintenanceDate = now;
                }

                var jobs = await _jobAppService.GetAllJobsAsync();

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
                await _activityService.LogActivityAsync(string.Format(_localizationService["SchedulerLoopError"], ex.Message), "ERROR");
                await Task.Delay(5000, cancellationToken);
            }

            await Task.Delay(TimeSpan.FromSeconds(1), cancellationToken);
        }
    }

    private async Task RunDailyMaintenanceAsync()
    {
        try
        {
            // 1. Purge
            var settings = await _settingsStore.LoadAsync();
            await _auditService.PurgeOldLogsAsync(settings.LogRetentionDays);

            // 2. Summary Email
            if (settings.EnableSmtp && !string.IsNullOrEmpty(settings.NotificationEmail))
            {
                var summary = await _auditService.GetDailySummaryAsync();
                await _notificationService.NotifyJobCompletionAsync(
                    new Job { Name = _localizationService["DailySummaryName"] }, true, 0, summary);
            }

            await _activityService.LogActivityAsync(_localizationService["DailyMaintenanceSuccess"]);
        }
        catch (Exception ex)
        {
            await _activityService.LogActivityAsync(string.Format(_localizationService["DailyMaintenanceError"], ex.Message), "WARNING");
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
                       if (diff.TotalDays >= 1) remaining = string.Format(_localizationService["InXDays"], (int)diff.TotalDays);
                       else if (diff.TotalHours >= 1) remaining = string.Format(_localizationService["InXHours"], (int)diff.TotalHours);
                       else remaining = string.Format(_localizationService["InXMinutes"], (int)diff.TotalMinutes);

                       return new UpcomingJobInfo {
                           JobName = x.Job.Name,
                           NextRun = x.Next.Value,
                           TimeRemaining = remaining
                       };
                   })
                   .ToList();
    }
}
