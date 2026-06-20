using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using AutoFlow.Domain.Entities;

namespace AutoFlow.Application.Interfaces;

public enum FileViewMode
{
    Grid,
    List,
    Details
}

public interface IFileExplorerService
{
    Task<IEnumerable<FileSystemItem>> GetDirectoryContentsAsync(string path, bool bypassAuth = false);
    Task<IEnumerable<FileSystemItem>> SearchAsync(string path, string query);
    Task<IEnumerable<FileSystemItem>> FilterByExtensionAsync(string path, IEnumerable<string> extensions);
    bool IsPathAuthorized(string path);
    Task<string?> GetParentPathAsync(string path);
    Task<IEnumerable<string>> GetAuthorizedRootsAsync();
}