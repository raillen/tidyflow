using System.Threading.Tasks;
using AutoFlow.Domain.Entities;

namespace AutoFlow.Application.Interfaces;

public interface IExternalNotificationService
{
    Task NotifyJobCompletionAsync(Job job, bool success, int processedFiles, string? errorMessage = null);
}
