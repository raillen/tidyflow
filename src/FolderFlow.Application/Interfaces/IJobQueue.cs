using System.Threading.Tasks;
using FolderFlow.Domain.Entities;

namespace FolderFlow.Application.Interfaces;

public interface IJobQueue
{
    bool IsPaused { get; set; }
    int Count { get; }
    ValueTask EnqueueAsync(Job job);
    ValueTask<Job> DequeueAsync();
}
