using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using AutoFlow.Domain.Entities;

namespace AutoFlow.Application.Interfaces;

public interface IAuthorizedPathProvider
{
    Task<IEnumerable<string>> GetAuthorizedPathsAsync();
    Task<IEnumerable<Blueprint>> GetAuthorizedBlueprintsAsync();
    Task<IEnumerable<Job>> GetAuthorizedJobsAsync();
    bool IsPathAuthorized(string path);
}