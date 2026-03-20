using System;
using System.IO;
using System.Security.Cryptography;
using System.Threading.Tasks;
using FolderFlow.Application.Interfaces;

namespace FolderFlow.Infrastructure.Filesystem;

public class Sha256HashService : IHashService
{
    public async Task<string> ComputeHashAsync(string filePath)
    {
        if (!File.Exists(filePath)) return string.Empty;

        using var sha256 = SHA256.Create();
        using var stream = File.OpenRead(filePath);
        var hashBytes = await sha256.ComputeHashAsync(stream);
        return BitConverter.ToString(hashBytes).Replace("-", "").ToLowerInvariant();
    }
}
