using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;

namespace FolderFlow.Application.Interfaces;

public record FileMetadata(long Size, DateTime LastWriteTimeUtc);

public interface IFileOperator
{
    Task CopyAsync(string source, string target, CancellationToken cancellationToken = default, IProgress<double>? progress = null);
    Task MoveAsync(string source, string target, CancellationToken cancellationToken = default);
    Task DeleteAsync(string path, CancellationToken cancellationToken = default);
    bool Exists(string path);
    IEnumerable<string> EnumerateFiles(string path, string searchPattern, bool recursive);
    void CreateDirectory(string path);
    FileMetadata? GetFileMetadata(string path);
}
