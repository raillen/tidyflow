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

    public async Task ProcessBlueprintAsync(Blueprint blueprint)
    {
        if (!blueprint.IsActive || !Directory.Exists(blueprint.Path)) return;

        try
        {
            // 1. Scaffolding (Pastas)
            if (blueprint.BlueprintFolders.Any())
            {
                var subDirs = Directory.GetDirectories(blueprint.Path);
                foreach (var dir in subDirs)
                {
                    await ApplyScaffoldingAsync(blueprint, dir);
                }
            }

            // 2. Renaming (Arquivos)
            if (!string.IsNullOrWhiteSpace(blueprint.RenameTemplate))
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
        return Task.Run(() =>
        {
            try
            {
                foreach (var subName in blueprint.BlueprintFolders)
                {
                    var targetSub = Path.Combine(newFolderPath, subName);
                    if (!Directory.Exists(targetSub))
                    {
                        Directory.CreateDirectory(targetSub);
                        _logger.LogAsync($"Organization: Blueprint Scaffolding created '{subName}' in '{Path.GetFileName(newFolderPath)}'", "INFO");
                    }
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
