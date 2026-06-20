using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using AutoFlow.Application.Interfaces;
using AutoFlow.Domain.Entities;

namespace AutoFlow.Infrastructure;

public class AuthorizedPathProvider : IAuthorizedPathProvider
{
    private readonly IBlueprintStore _blueprintStore;
    private readonly IJobStore _jobStore;
    private HashSet<string>? _cachedPaths;

    public AuthorizedPathProvider(IBlueprintStore blueprintStore, IJobStore jobStore)
    {
        _blueprintStore = blueprintStore;
        _jobStore = jobStore;
    }

    public async Task<IEnumerable<string>> GetAuthorizedPathsAsync()
    {
        await EnsureCacheAsync();
        return _cachedPaths!;
    }

    public async Task<IEnumerable<Blueprint>> GetAuthorizedBlueprintsAsync()
    {
        return await _blueprintStore.GetAllAsync();
    }

    public async Task<IEnumerable<Job>> GetAuthorizedJobsAsync()
    {
        return await _jobStore.GetAllAsync();
    }

    public bool IsPathAuthorized(string path)
    {
        if (_cachedPaths == null)
        {
            return false;
        }

        return _cachedPaths.Any(authorizedPath => 
            path.StartsWith(authorizedPath, StringComparison.OrdinalIgnoreCase));
    }

    private async Task EnsureCacheAsync()
    {
        if (_cachedPaths != null)
        {
            return;
        }

        var paths = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
        var blueprints = await _blueprintStore.GetAllAsync();
        var jobs = await _jobStore.GetAllAsync();

        foreach (var bp in blueprints)
        {
            if (!string.IsNullOrWhiteSpace(bp.Path) && Directory.Exists(bp.Path))
            {
                paths.Add(bp.Path);
            }
        }

        foreach (var job in jobs)
        {
            if (!string.IsNullOrWhiteSpace(job.SourcePath) && Directory.Exists(job.SourcePath))
            {
                paths.Add(job.SourcePath);
            }
            if (!string.IsNullOrWhiteSpace(job.TargetPath) && Directory.Exists(job.TargetPath))
            {
                paths.Add(job.TargetPath);
            }
        }

        _cachedPaths = paths;
    }

    public void InvalidateCache()
    {
        _cachedPaths = null;
    }
}