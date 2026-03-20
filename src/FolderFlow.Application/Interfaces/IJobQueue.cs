using System.Threading.Tasks;
using FolderFlow.Domain.Entities;

namespace FolderFlow.Application.Interfaces;

public interface IJobQueue
{
    ValueTask EnqueueAsync(Job job);
    ValueTask<Job> DequeueAsync();
}
