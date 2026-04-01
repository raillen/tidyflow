using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;
using System.Threading;
using System.Threading.Tasks;
using FolderFlow.Application.Interfaces;
using FolderFlow.Infrastructure.Security;
using Renci.SshNet;

namespace FolderFlow.Infrastructure.Filesystem;

public class SftpFileOperator : IFileOperator
{
    private readonly string _host;
    private readonly int _port;
    private readonly string _username;
    private readonly string _password;

    public long BandwidthLimit { get; set; }

    public SftpFileOperator(string host, int port, string username, string password)
    {
        _host = host;
        _port = port;
        _username = username;
        _password = password;
    }

    private SftpClient GetClient()
    {
        var client = new SftpClient(_host, _port, _username, _password);
        client.Connect();
        return client;
    }

    public async Task CopyAsync(string source, string target, CancellationToken cancellationToken = default, IProgress<double>? progress = null, string? encryptionKey = null, bool deltaSync = false)
    {
        bool sourceIsSftp = source.StartsWith("sftp://");
        bool targetIsSftp = target.StartsWith("sftp://");

        if (sourceIsSftp && targetIsSftp)
        {
            throw new NotSupportedException("A cópia direta de SFTP para SFTP ainda não é suportada.");
        }

        using var client = GetClient();

        if (sourceIsSftp && !targetIsSftp)
        {
            // Download do SFTP para Local
            var sftpPath = ExtractPath(source);
            using var remoteStream = client.OpenRead(sftpPath);
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
                    await remoteStream.CopyToAsync(targetStream, 81920, cancellationToken);
                    return;
                }

                await TransferWithProgressAsync(remoteStream, targetStream, remoteStream.Length, progress, cancellationToken);
            }
            finally
            {
                if (targetStream != targetFileStream)
                {
                    await targetStream.DisposeAsync();
                }
            }
        }
        else if (!sourceIsSftp && targetIsSftp)
        {
            // Upload Local para SFTP
            var sftpPath = ExtractPath(target);
            await using var sourceStream = new FileStream(source, FileMode.Open, FileAccess.Read, FileShare.Read, 4096, true);
            using var remoteStream = client.OpenWrite(sftpPath);
            
            // Note: encryptionKey is typically for encrypting the target, which would mean we need to wrap the remoteStream
            Stream finalRemoteStream = remoteStream;
            if (!string.IsNullOrWhiteSpace(encryptionKey))
            {
                finalRemoteStream = EncryptionHelper.GetEncryptStream(remoteStream, encryptionKey);
            }

            try
            {
                if (progress == null)
                {
                    await sourceStream.CopyToAsync(finalRemoteStream, 81920, cancellationToken);
                    return;
                }

                await TransferWithProgressAsync(sourceStream, finalRemoteStream, sourceStream.Length, progress, cancellationToken);
            }
            finally
            {
                if (finalRemoteStream != remoteStream)
                {
                    await finalRemoteStream.DisposeAsync();
                }
            }
        }
    }

    private async Task TransferWithProgressAsync(Stream sourceStream, Stream targetStream, long totalBytes, IProgress<double> progress, CancellationToken cancellationToken)
    {
        var buffer = new byte[1024 * 1024]; // 1MB buffer
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
        // Simple implementation: Copy then Delete
        await CopyAsync(source, target, cancellationToken);
        await DeleteAsync(source, cancellationToken);
    }

    public Task DeleteAsync(string path, CancellationToken cancellationToken = default)
    {
        if (path.StartsWith("sftp://"))
        {
            using var client = GetClient();
            var sftpPath = ExtractPath(path);
            if (client.Exists(sftpPath))
            {
                client.Delete(sftpPath);
            }
            return Task.CompletedTask;
        }
        else
        {
            if (File.Exists(path)) File.Delete(path);
            return Task.CompletedTask;
        }
    }

    public bool Exists(string path)
    {
        if (path.StartsWith("sftp://"))
        {
            using var client = GetClient();
            return client.Exists(ExtractPath(path));
        }
        return File.Exists(path) || Directory.Exists(path);
    }

    public IEnumerable<string> EnumerateFiles(string path, string searchPattern, bool recursive)
    {
        if (!path.StartsWith("sftp://"))
        {
            return Directory.EnumerateFiles(path, searchPattern, recursive ? SearchOption.AllDirectories : SearchOption.TopDirectoryOnly);
        }

        using var client = GetClient();
        var sftpPath = ExtractPath(path);
        
        // Very basic wildcard to regex conversion
        var regexPattern = "^" + Regex.Escape(searchPattern).Replace("\\*", ".*").Replace("\\?", ".") + "$";
        var regex = new Regex(regexPattern, RegexOptions.IgnoreCase);

        var result = new List<string>();
        EnumerateRemoteFiles(client, sftpPath, regex, recursive, result, "sftp://" + _host + ":" + _port);
        return result;
    }

    private void EnumerateRemoteFiles(SftpClient client, string path, Regex regex, bool recursive, List<string> result, string basePathPrefix)
    {
        if (!client.Exists(path)) return;

        foreach (var file in client.ListDirectory(path))
        {
            if (file.Name == "." || file.Name == "..") continue;

            if (file.IsRegularFile && regex.IsMatch(file.Name))
            {
                result.Add(basePathPrefix + file.FullName);
            }
            else if (file.IsDirectory && recursive)
            {
                EnumerateRemoteFiles(client, file.FullName, regex, recursive, result, basePathPrefix);
            }
        }
    }

    public void CreateDirectory(string path)
    {
        if (path.StartsWith("sftp://"))
        {
            using var client = GetClient();
            var sftpPath = ExtractPath(path);
            
            if (!client.Exists(sftpPath))
            {
                // SFTP usually requires creating parent directories one by one if they don't exist
                CreateRemoteDirectoryRecursive(client, sftpPath);
            }
        }
        else
        {
            if (!Directory.Exists(path))
            {
                Directory.CreateDirectory(path);
            }
        }
    }

    private void CreateRemoteDirectoryRecursive(SftpClient client, string path)
    {
        if (string.IsNullOrEmpty(path) || path == "/") return;
        
        var parent = path.Substring(0, path.LastIndexOf('/'));
        if (string.IsNullOrEmpty(parent)) parent = "/";

        if (!client.Exists(parent))
        {
            CreateRemoteDirectoryRecursive(client, parent);
        }
        
        if (!client.Exists(path))
        {
            client.CreateDirectory(path);
        }
    }

    public FileMetadata? GetFileMetadata(string path)
    {
        if (path.StartsWith("sftp://"))
        {
            using var client = GetClient();
            var sftpPath = ExtractPath(path);
            if (client.Exists(sftpPath))
            {
                var attrs = client.GetAttributes(sftpPath);
                return new FileMetadata(attrs.Size, attrs.LastWriteTime);
            }
            return null;
        }

        if (!File.Exists(path)) return null;
        var info = new FileInfo(path);
        return new FileMetadata(info.Length, info.LastWriteTimeUtc);
    }

    private string ExtractPath(string sftpUrl)
    {
        // Example: sftp://host:port/path/to/file
        var uri = new Uri(sftpUrl);
        return uri.AbsolutePath;
    }
}
