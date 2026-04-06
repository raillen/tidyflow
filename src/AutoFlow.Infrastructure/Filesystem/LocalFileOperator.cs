using System.Collections.Generic;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using AutoFlow.Application.Interfaces;
using AutoFlow.Infrastructure.Security;

namespace AutoFlow.Infrastructure.Filesystem;

public class LocalFileOperator : IFileOperator
{
    public long BandwidthLimit { get; set; }

    public async Task CopyAsync(string source, string target, CancellationToken cancellationToken = default, IProgress<double>? progress = null, string? encryptionKey = null, bool deltaSync = false)
    {
        if (deltaSync && string.IsNullOrWhiteSpace(encryptionKey) && File.Exists(target))
        {
            // Simple Block-Level Delta Sync (No encryption support for delta yet)
            await DeltaCopyAsync(source, target, cancellationToken, progress);
            return;
        }

        await using var sourceFileStream = new FileStream(source, FileMode.Open, FileAccess.Read, FileShare.Read, 4096, true);
        
        // Aplica Throttling se configurado
        Stream sourceStream = BandwidthLimit > 0 
            ? new ThrottledStream(sourceFileStream, BandwidthLimit) 
            : sourceFileStream;

        await using var targetFileStream = new FileStream(target, FileMode.Create, FileAccess.Write, FileShare.None, 4096, true);
        
        Stream targetStream = targetFileStream;
        if (!string.IsNullOrWhiteSpace(encryptionKey))
        {
            targetStream = EncryptionHelper.GetEncryptStream(targetFileStream, encryptionKey);
        }

        try
        {
            if (progress == null)
            {
                await sourceStream.CopyToAsync(targetStream, 81920, cancellationToken);
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
        finally
        {
            if (targetStream != targetFileStream)
            {
                await targetStream.DisposeAsync();
            }
        }
    }

    private async Task DeltaCopyAsync(string source, string target, CancellationToken cancellationToken, IProgress<double>? progress)
    {
        var totalBytes = new FileInfo(source).Length;
        long totalRead = 0;

        await using var rawSourceStream = new FileStream(source, FileMode.Open, FileAccess.Read, FileShare.Read, 4096, true);
        
        // Aplica Throttling
        Stream sourceStream = BandwidthLimit > 0 
            ? new ThrottledStream(rawSourceStream, BandwidthLimit) 
            : rawSourceStream;

        await using var targetStream = new FileStream(target, FileMode.OpenOrCreate, FileAccess.ReadWrite, FileShare.None, 4096, true);

        // If target is larger, truncate it to match source size
        if (targetStream.Length > totalBytes)
        {
            targetStream.SetLength(totalBytes);
        }

        var sourceBuffer = new byte[1024 * 1024]; // 1MB block
        var targetBuffer = new byte[1024 * 1024];

        int sourceBytesRead;
        while ((sourceBytesRead = await sourceStream.ReadAsync(sourceBuffer, 0, sourceBuffer.Length, cancellationToken)) > 0)
        {
            long currentPosition = targetStream.Position;
            int targetBytesRead = await targetStream.ReadAsync(targetBuffer, 0, sourceBytesRead, cancellationToken);

            bool isMatch = targetBytesRead == sourceBytesRead;
            if (isMatch)
            {
                for (int i = 0; i < sourceBytesRead; i++)
                {
                    if (sourceBuffer[i] != targetBuffer[i])
                    {
                        isMatch = false;
                        break;
                    }
                }
            }

            if (!isMatch)
            {
                targetStream.Position = currentPosition;
                await targetStream.WriteAsync(sourceBuffer, 0, sourceBytesRead, cancellationToken);
            }

            totalRead += sourceBytesRead;

            if (totalBytes > 0)
            {
                progress?.Report((double)totalRead / totalBytes * 100);
            }
        }
    }

    public async Task MoveAsync(string source, string target, CancellationToken cancellationToken = default)
    {
        // Detecta se  o mesmo volume (Disco)
        var sourceDrive = Path.GetPathRoot(Path.GetFullPath(source));
        var targetDrive = Path.GetPathRoot(Path.GetFullPath(target));

        if (sourceDrive == targetDrive)
        {
            // Mesmo disco: Operao atmica instantnea do SO
            await Task.Run(() => File.Move(source, target, true), cancellationToken);
        }
        else
        {
            // Discos diferentes: Fallback para Copy + Delete (permite reportar progresso no futuro se passarmos o IProgress)
            await CopyAsync(source, target, cancellationToken);
            
            // Verificao simples de sucesso antes de deletar a origem
            if (File.Exists(target))
            {
                var sInfo = new FileInfo(source);
                var tInfo = new FileInfo(target);
                if (sInfo.Length == tInfo.Length)
                {
                    File.Delete(source);
                }
                else
                {
                    throw new IOException("Falha na integridade do movimento entre volumes: Tamanhos divergem.");
                }
            }
        }
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
