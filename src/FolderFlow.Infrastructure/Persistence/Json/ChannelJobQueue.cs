using System.Threading.Channels;
using System.Threading.Tasks;
using FolderFlow.Application.Interfaces;
using FolderFlow.Domain.Entities;

namespace FolderFlow.Infrastructure.Persistence.Json; // Ou mover para .Infrastructure.Queuing

public class ChannelJobQueue : IJobQueue
{
    private readonly Channel<Job> _channel = Channel.CreateUnbounded<Job>();

    public ValueTask EnqueueAsync(Job job)
    {
        return _channel.Writer.WriteAsync(job);
    }

    public ValueTask<Job> DequeueAsync()
    {
        return _channel.Reader.ReadAsync();
    }
}
