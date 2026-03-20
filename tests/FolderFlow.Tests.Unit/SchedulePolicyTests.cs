using System;
using FolderFlow.Domain.Entities;
using FolderFlow.Domain.Enums;
using FolderFlow.Domain.Policies;
using Xunit;

namespace FolderFlow.Tests.Unit;

public class SchedulePolicyTests
{
    [Fact]
    public void ShouldRun_Interval_ReturnsTrue_WhenTimePassed()
    {
        // Arrange
        var job = new Job { ScheduleType = ScheduleType.Interval, IntervalMinutes = 10 };
        job.LastRun = DateTime.Now.AddMinutes(-11);
        var now = DateTime.Now;

        // Act
        var result = SchedulePolicy.ShouldRun(job, now);

        // Assert
        Assert.True(result);
    }

    [Fact]
    public void ShouldRun_Daily_ReturnsTrue_WhenTimeReached()
    {
        // Arrange
        var now = new DateTime(2026, 3, 20, 14, 30, 0);
        var job = new Job { ScheduleType = ScheduleType.Daily, ScheduleTime = "14:00" };
        job.LastRun = now.AddDays(-1);

        // Act
        var result = SchedulePolicy.ShouldRun(job, now);

        // Assert
        Assert.True(result);
    }

    [Fact]
    public void ShouldRun_Daily_ReturnsFalse_WhenAlreadyRunToday()
    {
        // Arrange
        var now = new DateTime(2026, 3, 20, 14, 30, 0);
        var job = new Job { ScheduleType = ScheduleType.Daily, ScheduleTime = "14:00" };
        job.LastRun = now.Date; // Rodou hoje cedo

        // Act
        var result = SchedulePolicy.ShouldRun(job, now);

        // Assert
        Assert.False(result);
    }
}
