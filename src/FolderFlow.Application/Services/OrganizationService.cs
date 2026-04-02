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

    public async Task ProcessOrganizationAsync(Job job)
    {
        if (!job.OrganizationEnabled || !Directory.Exists(job.SourcePath)) return;

        try
        {
            // 1. Scaffolding
            if (job.BlueprintFolders.Any())
            {
                var subDirs = Directory.GetDirectories(job.SourcePath);
                foreach (var dir in subDirs)
                {
                    await ApplyScaffoldingAsync(job, dir);
                }
            }

            // 2. Renaming (Simplificado: apenas arquivos no root da origem por enquanto para segurana)
            if (!string.IsNullOrWhiteSpace(job.RenameTemplate))
            {
                var files = Directory.GetFiles(job.SourcePath);
                foreach (var file in files)
                {
                    await TryRenameFileAsync(job, file);
                }
            }
        }
        catch (Exception ex)
        {
            await _logger.LogAsync($"Erro no processamento de organização: {ex.Message}", "ERROR");
        }
    }

    public Task ApplyScaffoldingAsync(Job job, string newFolderPath)
    {
        return Task.Run(() =>
        {
            try
            {
                foreach (var subName in job.BlueprintFolders)
                {
                    var targetSub = Path.Combine(newFolderPath, subName);
                    if (!Directory.Exists(targetSub))
                    {
                        Directory.CreateDirectory(targetSub);
                        _logger.LogAsync($"Organization: Scaffolding created '{subName}' in '{Path.GetFileName(newFolderPath)}'", "INFO");
                    }
                }
            }
            catch { }
        });
    }

    public Task<string> GetRenamedPathAsync(Job job, string originalPath)
    {
        if (string.IsNullOrWhiteSpace(job.RenameTemplate)) return Task.FromResult(originalPath);

        var fileName = Path.GetFileNameWithoutExtension(originalPath);
        var ext = Path.GetExtension(originalPath);
        var date = DateTime.Now.ToString("yyyy-MM-dd");
        var parent = Path.GetFileName(Path.GetDirectoryName(originalPath)) ?? "";

        var newName = job.RenameTemplate
            .Replace("{Date}", date)
            .Replace("{JobName}", job.Name)
            .Replace("{FileName}", fileName)
            .Replace("{Ext}", ext.TrimStart('.'))
            .Replace("{Parent}", parent);

        // Garante a extenso se no estiver no template
        if (!newName.EndsWith(ext, StringComparison.OrdinalIgnoreCase))
        {
            newName += ext;
        }

        var dir = Path.GetDirectoryName(originalPath);
        return Task.FromResult(Path.Combine(dir ?? "", newName));
    }

    private async Task TryRenameFileAsync(Job job, string filePath)
    {
        var newPath = await GetRenamedPathAsync(job, filePath);
        
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
