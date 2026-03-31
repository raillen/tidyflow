using System;
using System.IO;
using System.Runtime.InteropServices;
using System.Threading;
using FolderFlow.Application.Interfaces;

namespace FolderFlow.Infrastructure.Filesystem;

public class WindowsCloudHydrationService : ICloudHydrationService
{
    public async Task EnsureFileIsLocalAsync(string filePath, CancellationToken cancellationToken = default)
    {
        if (!OperatingSystem.IsWindows())
            return; // Suportado principalmente no Windows para OneDrive/SharePoint

        if (!File.Exists(filePath))
            return;

        try
        {
            var attributes = File.GetAttributes(filePath);

            // Verifica se o arquivo é um placeholder (ReparsePoint) ou se está offline
            bool isOffline = (attributes & FileAttributes.Offline) == FileAttributes.Offline;
            bool isReparsePoint = (attributes & FileAttributes.ReparsePoint) == FileAttributes.ReparsePoint;

            if (isOffline || isReparsePoint)
            {
                // Força o sistema operacional a baixar o arquivo da nuvem (hidratação).
                // Ao abrir o arquivo para leitura, o driver do OneDrive/SharePoint intercepta
                // e faz o download. Lemos o primeiro byte para garantir.
                await using (var fs = new FileStream(filePath, FileMode.Open, FileAccess.Read, FileShare.ReadWrite, 4096, true))
                {
                    var buffer = new byte[1];
                    await fs.ReadExactlyAsync(buffer, 0, 1, cancellationToken);
                }

                // Damos um tempo para o SO atualizar os atributos e sincronizar o arquivo
                // Loop de polling para garantir que o arquivo não está mais "Offline"
                int attempts = 0;
                while (attempts < 10) // Timeout de ~5 segundos
                {
                    await Task.Delay(500, cancellationToken);
                    attributes = File.GetAttributes(filePath);
                    isOffline = (attributes & FileAttributes.Offline) == FileAttributes.Offline;
                    
                    if (!isOffline) break;
                    attempts++;
                }
            }
        }
        catch (Exception)
        {
            // Se falhar ao hidratar, o ExecutionEngine vai capturar no momento da cópia.
            // Poderíamos logar aqui, mas optamos por manter simples.
        }
    }
}
