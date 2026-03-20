using System;
using System.IO;
using System.Runtime.InteropServices;
using System.Threading;
using FolderFlow.Application.Interfaces;

namespace FolderFlow.Infrastructure.Filesystem;

public class WindowsCloudHydrationService : ICloudHydrationService
{
    public void EnsureFileIsLocal(string filePath)
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
                using (var fs = new FileStream(filePath, FileMode.Open, FileAccess.Read, FileShare.ReadWrite))
                {
                    fs.ReadByte();
                }

                // Damos um tempo mínimo para o SO atualizar o status (opcional, mas recomendado)
                Thread.Sleep(500);
            }
        }
        catch (Exception)
        {
            // Se falhar ao hidratar, o ExecutionEngine vai capturar no momento da cópia.
            // Poderíamos logar aqui, mas optamos por manter simples.
        }
    }
}
