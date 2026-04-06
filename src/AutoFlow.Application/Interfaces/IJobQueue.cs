using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using AutoFlow.Domain.Entities;

namespace AutoFlow.Application.Interfaces;

public interface IJobQueue
{
    bool IsPaused { get; set; }
    int Count { get; }
    IEnumerable<Job> PendingJobs { get; }
    
    ValueTask EnqueueAsync(Job job);
    ValueTask<Job?> DequeueAsync();
    
    void Remove(Guid jobId);
    void MoveUp(Guid jobId);
    void MoveDown(Guid jobId);
    void PushToTop(Guid jobId);
    void Clear();
}
