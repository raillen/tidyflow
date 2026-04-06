using System;
using System.Threading.Tasks;
using AutoFlow.Domain.Entities;

namespace AutoFlow.Application.Interfaces;

public interface IRollbackStore
{
    Task SaveManifestAsync(RollbackManifest manifest);
    Task<RollbackManifest?> GetLatestManifestAsync(Guid jobId);
    Task ClearManifestAsync(Guid jobId);
}
