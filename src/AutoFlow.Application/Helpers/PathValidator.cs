using System;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;

namespace AutoFlow.Application.Helpers;

public static class PathValidator
{
    private static readonly char[] InvalidFileNameChars = Path.GetInvalidFileNameChars();
    private static readonly string[] ReservedNames = 
    { 
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9" 
    };

    public static bool IsValidPath(string path, out string errorMessage)
    {
        errorMessage = string.Empty;

        if (string.IsNullOrWhiteSpace(path))
        {
            errorMessage = "O caminho não pode estar vazio.";
            return false;
        }

        try
        {
            // 1. Prevenção de Path Traversal (..)
            if (path.Contains("..") || path.Contains("./") || path.Contains(".\\"))
            {
                errorMessage = "Sequências de escape de diretório (..) não são permitidas por segurança.";
                return false;
            }

            // 2. Verificação de caracteres inválidos (exceto os de navegação de path no Windows como : e \)
            // Vamos validar os nomes de pastas/arquivos individualmente
            var components = path.Split(new[] { Path.DirectorySeparatorChar, Path.AltDirectorySeparatorChar }, StringSplitOptions.RemoveEmptyEntries);
            
            // Pular a letra do drive (ex: C:) se existir
            int startIndex = (path.Length >= 2 && path[1] == ':') ? 1 : 0;

            for (int i = startIndex; i < components.Length; i++)
            {
                var part = components[i];

                // Caracteres proibidos no Windows
                if (part.Any(c => InvalidFileNameChars.Contains(c)))
                {
                    errorMessage = $"O componente '{part}' contém caracteres inválidos para o Windows.";
                    return false;
                }

                // Nomes reservados do sistema
                if (ReservedNames.Contains(part.ToUpperInvariant()))
                {
                    errorMessage = $"O nome '{part}' é reservado pelo sistema e não pode ser usado.";
                    return false;
                }
            }

            // 3. Normalização e Verificação de Root
            var fullPath = Path.GetFullPath(path);
            
            return true;
        }
        catch (Exception ex)
        {
            errorMessage = $"Caminho inválido: {ex.Message}";
            return false;
        }
    }
}
