using System;
using System.Linq;
using FolderFlow.Domain.Entities;
using FolderFlow.Domain.Enums;

namespace FolderFlow.Domain.Policies;

public static class SchedulePolicy
{
    public static bool ShouldRun(Job job, DateTime now)
    {
        if (job.ScheduleType == ScheduleType.None) return false;

        // Se tem data especifica, s roda se for HOJE e a hora bateu
        if (job.SpecificDate.HasValue && job.SpecificDate.Value.Date != now.Date) return false;

        if (job.ScheduleType == ScheduleType.Interval)
        {
            if (job.LastRun == null) return true;
            return (now - job.LastRun.Value).TotalMinutes >= job.IntervalMinutes;
        }

        if (job.ScheduleType == ScheduleType.Daily)
        {
            if (string.IsNullOrEmpty(job.ScheduleTime)) return false;
            
            // Se j rodou hoje, no roda de novo
            if (job.LastRun != null && job.LastRun.Value.Date == now.Date) return false;

            if (TimeSpan.TryParse(job.ScheduleTime, out var scheduledTime))
            {
                return now.TimeOfDay >= scheduledTime;
            }
        }

        if (job.ScheduleType == ScheduleType.Weekly)
        {
            if (string.IsNullOrEmpty(job.ScheduleTime)) return false;
            
            // Se no definiu dias, assume todos (Daily) ou bloqueia? Vamos assumir que precisa estar na lista
            if (job.DaysOfWeek != null && job.DaysOfWeek.Any() && !job.DaysOfWeek.Contains(now.DayOfWeek)) return false;

            // Se j rodou hoje, no roda de novo
            if (job.LastRun != null && job.LastRun.Value.Date == now.Date) return false;

            if (TimeSpan.TryParse(job.ScheduleTime, out var scheduledTime))
            {
                return now.TimeOfDay >= scheduledTime;
            }
        }

        return false;
    }

    public static DateTime? GetNextRun(Job job, DateTime now)
    {
        if (job.ScheduleType == ScheduleType.None) return null;

        if (job.ScheduleType == ScheduleType.Interval)
        {
            if (job.LastRun == null) return now;
            var next = job.LastRun.Value.AddMinutes(job.IntervalMinutes);
            return next < now ? now : next;
        }

        if (TimeSpan.TryParse(job.ScheduleTime, out var scheduledTime))
        {
            DateTime next;
            if (job.ScheduleType == ScheduleType.Daily)
            {
                next = now.Date.Add(scheduledTime);
                if (next < now || (job.LastRun != null && job.LastRun.Value.Date == now.Date)) 
                    next = next.AddDays(1);
                return next;
            }

            if (job.ScheduleType == ScheduleType.Weekly && job.DaysOfWeek != null && job.DaysOfWeek.Any())
            {
                next = now.Date.Add(scheduledTime);
                // Busca o prximo dia da semana permitido
                for (int i = 0; i < 8; i++)
                {
                    var checkDay = next.AddDays(i);
                    if (job.DaysOfWeek.Contains(checkDay.DayOfWeek))
                    {
                        var scheduledOnCheckDay = checkDay.Date.Add(scheduledTime);
                        if (scheduledOnCheckDay > now && !(job.LastRun != null && job.LastRun.Value.Date == scheduledOnCheckDay.Date))
                            return scheduledOnCheckDay;
                    }
                }
            }
        }

        return null;
    }
}
