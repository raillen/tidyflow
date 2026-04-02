using System;
using FolderFlow.Domain.Entities;

namespace FolderFlow.Application.Interfaces;

public interface IWatchService : IDisposable
{
    void StartWatching(Job job, Action<Job> onFileChanged);
    void StopWatching(Job job);
    bool IsWatching(Job job);

    // Suporte a Blueprint (Novo)
    void StartWatchingBlueprint(Blueprint blueprint, Action<Blueprint> onChanged);
    void StopWatchingBlueprint(Blueprint blueprint);
    bool IsWatchingBlueprint(Blueprint blueprint);
}
