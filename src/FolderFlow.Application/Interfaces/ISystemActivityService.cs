using System;
using System.Collections.Generic;
using System.Threading.Tasks;

namespace FolderFlow.Application.Interfaces;

public record SystemActivity(DateTime Timestamp, string Message, string Level = "INFO");

public interface ISystemActivityService
{
    Task LogActivityAsync(string message, string level = "INFO");
    Task<IEnumerable<SystemActivity>> GetRecentActivitiesAsync(int count = 50);
}
