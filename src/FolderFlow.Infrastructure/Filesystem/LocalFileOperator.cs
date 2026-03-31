using System.Collections.Generic;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using FolderFlow.Application.Interfaces;

namespace FolderFlow.Infrastructure.Filesystem;

public class LocalFileOperator : IFileOperator
{
    public async Task CopyAsync(string source, string target, CancellationToken cancellationToken = default, IProgress<double>? progress = null)
    {
        await using var sourceStream = new FileStream(source, FileMode.Open, FileAccess.Read, FileShare.Read, 4096, true);
        await using var targetStream = new FileStream(target, FileMode.Create, FileAccess.Write, FileShare.None, 4096, true);
        
        if (progress == null)
        {
            await sourceStream.CopyToAsync(targetStream, cancellationToken);
            return;
        }

        var totalBytes = sourceStream.Length;
        var buffer = new byte[1024 * 1024]; // 1MB buffer for better performance and speed tracking
        long totalRead = 0;
        int bytesRead;

        while ((bytesRead = await sourceStream.ReadAsync(buffer, 0, buffer.Length, cancellationToken)) > 0)
        {
            await targetStream.WriteAsync(buffer, 0, bytesRead, cancellationToken);
            totalRead += bytesRead;
            
            if (totalBytes > 0)
            {
                progress.Report((double)totalRead / totalBytes * 100);
            }
        }
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
