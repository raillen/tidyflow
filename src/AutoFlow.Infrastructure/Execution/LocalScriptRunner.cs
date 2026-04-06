using System;
using System.Diagnostics;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using AutoFlow.Application.Interfaces;

namespace AutoFlow.Infrastructure.Execution;

public class LocalScriptRunner : IScriptRunner
{
    private readonly IAppLogger _logger;

    public LocalScriptRunner(IAppLogger logger)
    {
        _logger = logger;
    }

    public async Task<bool> RunScriptAsync(string scriptPath, string arguments = "", CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(scriptPath) || !File.Exists(scriptPath))
        {
            await _logger.LogAsync($"Script não encontrado ou não configurado: {scriptPath}", "WARNING");
            return false;
        }

        try
        {
            await _logger.LogAsync($"Executando hook (script): {scriptPath} com argumentos: {arguments}");

            var startInfo = new ProcessStartInfo
            {
                FileName = scriptPath,
                Arguments = arguments,
                UseShellExecute = false, // Permite redirecionar a sada
                CreateNoWindow = true,
                RedirectStandardOutput = true,
                RedirectStandardError = true
            };

            // Se for .ps1, precisamos rodar pelo powershell
            if (scriptPath.EndsWith(".ps1", StringComparison.OrdinalIgnoreCase))
            {
                startInfo.FileName = "powershell.exe";
                // Passa o arquivo e em seguida os argumentos
                startInfo.Arguments = $"-NoProfile -ExecutionPolicy Bypass -File \"{scriptPath}\" {arguments}";
            }

            using var process = Process.Start(startInfo);
            if (process == null) return false;

            await process.WaitForExitAsync(cancellationToken);

            var output = await process.StandardOutput.ReadToEndAsync(cancellationToken);
            var error = await process.StandardError.ReadToEndAsync(cancellationToken);

            if (!string.IsNullOrWhiteSpace(output))
                await _logger.LogAsync($"Script Output: {output}");
            
            if (process.ExitCode != 0 || !string.IsNullOrWhiteSpace(error))
            {
                await _logger.LogAsync($"Script falhou (ExitCode {process.ExitCode}): {error}", "ERROR");
                return false;
            }

            return true;
        }
        catch (OperationCanceledException)
        {
            await _logger.LogAsync($"Execuo de script cancelada: {scriptPath}", "WARNING");
            return false;
        }
        catch (Exception ex)
        {
            await _logger.LogAsync($"Erro ao executar script '{scriptPath}': {ex.Message}", "ERROR");
            return false;
        }
    }
}
