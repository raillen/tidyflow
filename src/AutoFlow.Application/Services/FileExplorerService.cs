using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using AutoFlow.Application.Interfaces;
using AutoFlow.Domain.Entities;

namespace AutoFlow.Application.Services;

public class FileExplorerService : IFileExplorerService
{
    private readonly IFileOperator _fileOperator;
    private readonly IAuthorizedPathProvider _pathProvider;
    private readonly IAppLogger _logger;

    public FileExplorerService(
        IFileOperator fileOperator,
        IAuthorizedPathProvider pathProvider,
        IAppLogger logger)
    {
        _fileOperator = fileOperator;
        _pathProvider = pathProvider;
        _logger = logger;
    }

    public async Task<IEnumerable<FileSystemItem>> GetDirectoryContentsAsync(string path, bool bypassAuth = false)
    {
        if (!bypassAuth && !_pathProvider.IsPathAuthorized(path))
        {
            await _logger.LogAsync($"Acesso não autorizado: {path}", "WARNING");
            return Enumerable.Empty<FileSystemItem>();
        }

        if (!Directory.Exists(path))
        {
            return Enumerable.Empty<FileSystemItem>();
        }

        var items = new List<FileSystemItem>();

        try
        {
            var dirInfo = new DirectoryInfo(path);

            foreach (var dir in dirInfo.GetDirectories())
            {
                items.Add(new FileSystemItem
                {
                    Name = dir.Name,
                    FullPath = dir.FullName,
                    IsDirectory = true,
                    ModifiedAt = dir.LastWriteTime,
                    IsHidden = (dir.Attributes & FileAttributes.Hidden) != 0,
                    IsReadOnly = (dir.Attributes & FileAttributes.ReadOnly) != 0
                });
            }

            foreach (var file in dirInfo.GetFiles())
            {
                items.Add(new FileSystemItem
                {
                    Name = file.Name,
                    FullPath = file.FullName,
                    IsDirectory = false,
                    Size = file.Length,
                    ModifiedAt = file.LastWriteTime,
                    Extension = file.Extension,
                    IsHidden = (file.Attributes & FileAttributes.Hidden) != 0,
                    IsReadOnly = (file.Attributes & FileAttributes.ReadOnly) != 0
                });
            }
        }
        catch (Exception ex)
        {
            await _logger.LogAsync($"Erro ao ler diretório {path}: {ex.Message}", "ERROR");
        }

        return items.OrderBy(x => !x.IsDirectory).ThenBy(x => x.Name);
    }

    public async Task<IEnumerable<FileSystemItem>> SearchAsync(string path, string query)
    {
        if (string.IsNullOrWhiteSpace(query) || query.Length < 2)
        {
            return await GetDirectoryContentsAsync(path, true);
        }

        var contents = await GetDirectoryContentsAsync(path, true);
        var lowerQuery = query.ToLowerInvariant();

        return contents.Where(x => x.Name.ToLowerInvariant().Contains(lowerQuery));
    }

    public async Task<IEnumerable<FileSystemItem>> FilterByExtensionAsync(
        string path,
        IEnumerable<string> extensions)
    {
        var contents = await GetDirectoryContentsAsync(path, true);
        var extList = extensions.Select(x => x.ToLowerInvariant()).ToHashSet();

        return contents.Where(x => 
            !x.IsDirectory && 
            x.Extension != null && 
            extList.Contains(x.Extension.ToLowerInvariant()));
    }

    public bool IsPathAuthorized(string path)
    {
        return _pathProvider.IsPathAuthorized(path);
    }

    public Task<string?> GetParentPathAsync(string path)
    {
        var parent = Directory.GetParent(path)?.FullName;
        return Task.FromResult(parent);
    }

    public async Task<IEnumerable<string>> GetAuthorizedRootsAsync()
    {
        return await _pathProvider.GetAuthorizedPathsAsync();
    }
}