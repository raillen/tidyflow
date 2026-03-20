using System;
using System.Collections.Generic;
using System.Threading.Tasks;

namespace FolderFlow.Application.Interfaces;

public interface IFailureStore
{
    Task SaveFailuresAsync(Guid jobId, IEnumerable<string> failedPaths);
    Task<IEnumerable<string>> GetFailuresAsync(Guid jobId);
    Task ClearFailuresAsync(Guid jobId);
}
