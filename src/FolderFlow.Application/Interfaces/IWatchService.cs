using System;
using FolderFlow.Domain.Entities;

namespace FolderFlow.Application.Interfaces;

public interface IWatchService : IDisposable
{
    void StartWatching(Job job, Action<Job> onFileChanged);
    void StopWatching(Job job);
    bool IsWatching(Job job);
}
