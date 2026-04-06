using System;
using AutoFlow.Domain.Entities;

namespace AutoFlow.Application.Interfaces;

public interface IWatchService : IDisposable
{
    void StartWatching(Job job, Action<Job> onFileChanged);
    void StopWatching(Job job);
    bool IsWatching(Job job);

    // Suporte a Blueprint Reativo (Atualizado)
    void StartWatchingBlueprint(Blueprint blueprint, Action<Blueprint, string> onChanged);
    void StopWatchingBlueprint(Blueprint blueprint);
    bool IsWatchingBlueprint(Blueprint blueprint);
}
