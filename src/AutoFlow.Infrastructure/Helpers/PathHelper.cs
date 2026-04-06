using System;
using System.IO;
using System.Runtime.InteropServices;

namespace AutoFlow.Infrastructure.Helpers;

public static class PathHelper
{
    public static string NormalizePath(string path)
    {
        if (string.IsNullOrWhiteSpace(path)) return path;

        // Somente Windows precisa do prefixo \\?\ para caminhos longos
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
        {
            var fullPath = Path.GetFullPath(path);
            if (fullPath.Length >= 250 && !fullPath.StartsWith(@"\\?\"))
            {
                return @"\\?\" + fullPath;
            }
            return fullPath;
        }

        return path;
    }
}
