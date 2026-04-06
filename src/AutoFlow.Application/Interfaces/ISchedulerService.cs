using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using AutoFlow.Domain.Entities;

namespace AutoFlow.Application.Interfaces;

public interface ISchedulerService
{
    void Start(CancellationToken cancellationToken);
    Task<IEnumerable<UpcomingJobInfo>> GetUpcomingJobsAsync(int count = 5);
}

public class UpcomingJobInfo
{
    public string JobName { get; set; } = string.Empty;
    public DateTime NextRun { get; set; }
    public string TimeRemaining { get; set; } = string.Empty;
}
