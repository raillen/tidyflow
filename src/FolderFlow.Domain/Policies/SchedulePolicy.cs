using System;
using FolderFlow.Domain.Entities;
using FolderFlow.Domain.Enums;

namespace FolderFlow.Domain.Policies;

public static class SchedulePolicy
{
    public static bool ShouldRun(Job job, DateTime now)
    {
        if (job.ScheduleType == ScheduleType.None) return false;

        if (job.ScheduleType == ScheduleType.Interval)
        {
            if (job.LastRun == null) return true;
            return (now - job.LastRun.Value).TotalMinutes >= job.IntervalMinutes;
        }

        if (job.ScheduleType == ScheduleType.Daily)
        {
            if (string.IsNullOrEmpty(job.ScheduleTime)) return false;
            
            // Se já rodou hoje, não roda de novo
            if (job.LastRun != null && job.LastRun.Value.Date == now.Date) return false;

            if (TimeSpan.TryParse(job.ScheduleTime, out var scheduledTime))
            {
                // Se a hora atual já passou da hora agendada
                return now.TimeOfDay >= scheduledTime;
            }
        }

        return false;
    }
}
