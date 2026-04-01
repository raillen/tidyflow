using System.Threading.Tasks;
using FolderFlow.Domain.Entities;

namespace FolderFlow.Application.Interfaces;

public interface IExternalNotificationService
{
    Task NotifyJobCompletionAsync(Job job, bool success, int processedFiles, string? errorMessage = null);
}
