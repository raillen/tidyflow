using System.Threading.Channels;
using System.Threading.Tasks;
using FolderFlow.Application.Interfaces;
using FolderFlow.Domain.Entities;

namespace FolderFlow.Infrastructure.Persistence.Json; // Ou mover para .Infrastructure.Queuing

public class ChannelJobQueue : IJobQueue
{
    private readonly Channel<Job> _channel = Channel.CreateUnbounded<Job>();
    private int _count;

    public bool IsPaused { get; set; }
    public int Count => _count;

    public async ValueTask EnqueueAsync(Job job)
    {
        await _channel.Writer.WriteAsync(job);
        System.Threading.Interlocked.Increment(ref _count);
    }

    public async ValueTask<Job> DequeueAsync()
    {
        var job = await _channel.Reader.ReadAsync();
        System.Threading.Interlocked.Decrement(ref _count);
        return job;
    }
}
