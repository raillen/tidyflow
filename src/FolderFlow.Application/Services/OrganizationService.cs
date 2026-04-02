using System;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using FolderFlow.Application.Interfaces;
using FolderFlow.Domain.Entities;

namespace FolderFlow.Application.Services;

public class OrganizationService : IOrganizationService
{
    private readonly IAppLogger _logger;

    public OrganizationService(IAppLogger logger)
    {
        _logger = logger;
    }

    public async Task ProcessBlueprintAsync(Blueprint blueprint, string? eventPath = null)
    {
        if (!blueprint.IsActive || !Directory.Exists(blueprint.Path)) return;

        try
        {
            // Se temos um eventPath, agimos apenas nele (Reatividade)
            if (!string.IsNullOrEmpty(eventPath))
            {
                if (Directory.Exists(eventPath))
                {
                    if (!blueprint.AutoScaffoldingEnabled) return;

                    // Evita criar subpastas dentro das subpastas do blueprint (Loop infinito)
                    if (blueprint.BlueprintFolders.Any(f => eventPath.EndsWith(f, StringComparison.OrdinalIgnoreCase))) 
                        return;

                    await ApplyScaffoldingAsync(blueprint, eventPath);
                }
                else if (File.Exists(eventPath))
                {
                    if (!blueprint.AutoRenamingEnabled) return;
                    await TryRenameFileAsync(blueprint.RenameTemplate ?? "", blueprint.Name, eventPath);
                }
                return;
            }

            // Fallback: Varredura completa (Execuo manual ou inicializao)
            // No modo manual, processamos ambos se estiverem habilitados
            
            // 1. Scaffolding (Pastas)
            if (blueprint.AutoScaffoldingEnabled && blueprint.BlueprintFolders.Any())
            {
                var subDirs = Directory.GetDirectories(blueprint.Path);
                foreach (var dir in subDirs)
                {
                    await ApplyScaffoldingAsync(blueprint, dir);
                }
            }

            // 2. Renaming (Arquivos)
            if (blueprint.AutoRenamingEnabled && !string.IsNullOrWhiteSpace(blueprint.RenameTemplate))
            {
                var files = Directory.GetFiles(blueprint.Path);
                foreach (var file in files)
                {
                    await TryRenameFileAsync(blueprint.RenameTemplate, blueprint.Name, file);
                }
            }
        }
        catch (Exception ex)
        {
            await _logger.LogAsync($"Organization: Error processing blueprint '{blueprint.Name}': {ex.Message}", "ERROR");
        }
    }

    public Task ApplyScaffoldingAsync(Blueprint blueprint, string newFolderPath)
    {
        return Task.Run(async () =>
        {
            try
            {
                bool createdAny = false;
                foreach (var subName in blueprint.BlueprintFolders)
                {
                    var targetSub = Path.Combine(newFolderPath, subName);
                    if (!Directory.Exists(targetSub))
                    {
                        Directory.CreateDirectory(targetSub);
                        createdAny = true;
                    }
                }
                if (createdAny)
                {
                    await _logger.LogAsync($"Organization: Blueprint Scaffolding applied to '{Path.GetFileName(newFolderPath)}'", "INFO");
                }
            }
            catch { }
        });
    }

    public Task<string> GetRenamedPathAsync(string renameTemplate, string jobName, string originalPath)
    {
        if (string.IsNullOrWhiteSpace(renameTemplate)) return Task.FromResult(originalPath);

        var fileName = Path.GetFileNameWithoutExtension(originalPath);
        var ext = Path.GetExtension(originalPath);
        var date = DateTime.Now.ToString("yyyy-MM-dd");
        var parent = Path.GetFileName(Path.GetDirectoryName(originalPath)) ?? "";

        var newName = renameTemplate
            .Replace("{Date}", date)
            .Replace("{JobName}", jobName)
            .Replace("{FileName}", fileName)
            .Replace("{Ext}", ext.TrimStart('.'))
            .Replace("{Parent}", parent);

        // Garante a extenso
        if (!newName.EndsWith(ext, StringComparison.OrdinalIgnoreCase))
        {
            newName += ext;
        }

        var dir = Path.GetDirectoryName(originalPath);
        return Task.FromResult(Path.Combine(dir ?? "", newName));
    }

    private async Task TryRenameFileAsync(string template, string name, string filePath)
    {
        if (string.IsNullOrWhiteSpace(template)) return;

        var newPath = await GetRenamedPathAsync(template, name, filePath);
        
        if (newPath != filePath && !File.Exists(newPath))
        {
            try
            {
                File.Move(filePath, newPath);
                await _logger.LogAsync($"Organization: Renamed '{Path.GetFileName(filePath)}' to '{Path.GetFileName(newPath)}'", "INFO");
            }
            catch (Exception ex)
            {
                await _logger.LogAsync($"Organization: Failed to rename '{Path.GetFileName(filePath)}': {ex.Message}", "WARNING");
            }
        }
    }
}
