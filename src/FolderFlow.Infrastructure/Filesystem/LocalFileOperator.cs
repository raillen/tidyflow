using System.Collections.Generic;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using FolderFlow.Application.Interfaces;

namespace FolderFlow.Infrastructure.Filesystem;

public class LocalFileOperator : IFileOperator
{
    public async Task CopyAsync(string source, string target, CancellationToken cancellationToken = default)
    {
        await using var sourceStream = new FileStream(source, FileMode.Open, FileAccess.Read, FileShare.Read, 4096, true);
        await using var targetStream = new FileStream(target, FileMode.Create, FileAccess.Write, FileShare.None, 4096, true);
        await sourceStream.CopyToAsync(targetStream, cancellationToken);
    }

    public async Task MoveAsync(string source, string target, CancellationToken cancellationToken = default)
    {
        // No Windows/Linux, File.Move é atômico se no mesmo volume. 
        // Para o MVP, usamos a API nativa do .NET.
        await Task.Run(() => File.Move(source, target, true), cancellationToken);
    }

    public Task DeleteAsync(string path, CancellationToken cancellationToken = default)
    {
        if (File.Exists(path))
        {
            File.Delete(path);
        }
        return Task.CompletedTask;
    }

    public bool Exists(string path) => File.Exists(path) || Directory.Exists(path);

    public IEnumerable<string> EnumerateFiles(string path, string searchPattern, bool recursive)
    {
        return Directory.EnumerateFiles(path, searchPattern, recursive ? SearchOption.AllDirectories : SearchOption.TopDirectoryOnly);
    }

    public void CreateDirectory(string path)
    {
        if (!Directory.Exists(path))
        {
            Directory.CreateDirectory(path);
        }
    }

    public FileMetadata? GetFileMetadata(string path)
    {
        if (!File.Exists(path)) return null;
        
        var info = new FileInfo(path);
        return new FileMetadata(info.Length, info.LastWriteTimeUtc);
    }
}
