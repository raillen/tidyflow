using System.IO;

namespace AutoFlow.Application.Interfaces;

public interface ICloudHydrationService
{
    /// <summary>
    /// Verifica se um arquivo é "Online-Only" (ex: OneDrive/SharePoint placeholder)
    /// e força o download para o disco local antes do processamento.
    /// </summary>
    Task EnsureFileIsLocalAsync(string filePath, CancellationToken cancellationToken = default);
}
