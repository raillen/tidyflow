using System;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;
using System.Threading.Tasks;
using System.Globalization;
using FolderFlow.Application.Interfaces;
using FolderFlow.Domain.Entities;
using FolderFlow.Domain.Enums;

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
            if (!string.IsNullOrEmpty(eventPath))
            {
                if (Directory.Exists(eventPath))
                {
                    // Pastas só acionam Blueprints do tipo Folder
                    if (blueprint.Type != BlueprintType.Folder) return;

                    // Scaffolding
                    if (blueprint.AutoScaffoldingEnabled && !blueprint.BlueprintFolders.Any(f => eventPath.EndsWith(f, StringComparison.OrdinalIgnoreCase)))
                    {
                        await ApplyScaffoldingAsync(blueprint, eventPath);
                    }

                    // Renomeio de Pasta
                    if (blueprint.AutoRenamingEnabled)
                    {
                        await TryRenameFolderAsync(blueprint, eventPath);
                    }
                }
                else if (File.Exists(eventPath))
                {
                    // Arquivos só acionam Blueprints do tipo File
                    if (blueprint.Type != BlueprintType.File) return;

                    if (blueprint.AutoRenamingEnabled)
                    {
                        await TryRenameFileAsync(blueprint.RenameTemplate ?? "", blueprint.Name, eventPath);
                    }
                }
                return;
            }

            // Processamento em lote (Batch)
            if (blueprint.Type == BlueprintType.Folder)
            {
                var subDirs = Directory.GetDirectories(blueprint.Path);
                foreach (var dir in subDirs)
                {
                    if (blueprint.AutoScaffoldingEnabled) await ApplyScaffoldingAsync(blueprint, dir);
                    if (blueprint.AutoRenamingEnabled) await TryRenameFolderAsync(blueprint, dir);
                }
            }
            else if (blueprint.Type == BlueprintType.File && blueprint.AutoRenamingEnabled)
            {
                var files = Directory.GetFiles(blueprint.Path);
                foreach (var file in files) await TryRenameFileAsync(blueprint.RenameTemplate ?? "", blueprint.Name, file);
            }
        }
        catch (Exception ex)
        {
            await _logger.LogAsync($"Organization: Error processing blueprint '{blueprint.Name}': {ex.Message}", "ERROR");
        }
    }

    public async Task ApplyScaffoldingAsync(Blueprint blueprint, string newFolderPath)
    {
        if (blueprint.Type != FolderFlow.Domain.Enums.BlueprintType.Folder) return;

        foreach (var sub in blueprint.BlueprintFolders)
        {
            var target = Path.Combine(newFolderPath, sub);
            if (!Directory.Exists(target))
            {
                try
                {
                    Directory.CreateDirectory(target);
                    await _logger.LogAsync($"Organization: Scaffolding created '{sub}' in '{Path.GetFileName(newFolderPath)}'", "DEBUG");
                }
                catch (Exception ex)
                {
                    await _logger.LogAsync($"Organization: Failed to create scaffolding folder '{sub}': {ex.Message}", "WARNING");
                }
            }
        }
    }

    private async Task TryRenameFolderAsync(Blueprint blueprint, string folderPath)
    {
        if (string.IsNullOrWhiteSpace(blueprint.RenameTemplate)) return;

        var dirName = Path.GetFileName(folderPath);
        var parentDir = Path.GetDirectoryName(folderPath) ?? "";
        
        var newName = await GetRenamedPathAsync(blueprint.RenameTemplate, blueprint.Name, folderPath, true);
        var newPath = Path.Combine(parentDir, Path.GetFileName(newName));

        if (newPath != folderPath)
        {
            if (Directory.Exists(newPath))
            {
                newPath = GenerateUniqueDirectoryName(newPath);
            }

            try
            {
                Directory.Move(folderPath, newPath);
                await _logger.LogAsync($"Organization: Folder renamed '{dirName}' to '{Path.GetFileName(newPath)}'", "INFO");
            }
            catch (Exception ex)
            {
                await _logger.LogAsync($"Organization: Failed to rename folder '{dirName}': {ex.Message}", "WARNING");
            }
        }
    }

    public Task<string> GetRenamedPathAsync(string renameTemplate, string jobName, string originalPath)
    {
        return GetRenamedPathAsync(renameTemplate, jobName, originalPath, false);
    }

    public Task<string> GetRenamedPathAsync(string renameTemplate, string jobName, string originalPath, bool isFolder)
    {
        return GetRenamedPathAsync(renameTemplate, jobName, originalPath, isFolder, 1);
    }

    public Task<string> GetRenamedPathAsync(string renameTemplate, string jobName, string originalPath, bool isFolder, int counterStart)
    {
        if (string.IsNullOrWhiteSpace(renameTemplate)) return Task.FromResult(originalPath);

        var nameOnly = isFolder ? Path.GetFileName(originalPath) : Path.GetFileNameWithoutExtension(originalPath);
        var ext = isFolder ? "" : Path.GetExtension(originalPath);
        var dir = Path.GetDirectoryName(originalPath) ?? "";
        
        var result = ProcessTemplate(renameTemplate, jobName, nameOnly, ext, dir, counterStart);

        if (!isFolder && !result.EndsWith(ext, StringComparison.OrdinalIgnoreCase))
        {
            result += ext;
        }

        return Task.FromResult(Path.Combine(dir, result));
    }

    private string ProcessTemplate(string template, string jobName, string fileName, string ext, string dir, int counterStart)
    {
        var now = DateTime.Now;
        
        // 1. Unifica atalhos para formato padrão {Counter:N} antes do processamento principal
        var normalizedTemplate = template.Replace("{01}", "{Counter:2}")
                                         .Replace("{001}", "{Counter:3}");

        // 2. Processa Token {Counter}
        // Suporta: {Counter}, {Counter:padding}
        var result = Regex.Replace(normalizedTemplate, @"\{Counter(?::(?<padding>\d+))?\}", m =>
        {
            int padding = 1;
            if (m.Groups["padding"].Success) int.TryParse(m.Groups["padding"].Value, out padding);
            
            int existingCount = Directory.Exists(dir) ? Directory.GetFileSystemEntries(dir).Length : 0;
            int finalNumber = counterStart + existingCount;
            
            return finalNumber.ToString().PadLeft(padding, '0');
        });

        // 3. Processa Todos os Tokens com Modificadores Avançados
        // Suporta: {Token}, {Token:modifier}, {Token:modifier(param)}
        result = Regex.Replace(result, @"\{(?<token>\w+)(?::(?<modifier>\w+)(?:\((?<params>.*?)\))?)?\}", m => 
        {
            var token = m.Groups["token"].Value;
            var modifier = m.Groups["modifier"].Success ? m.Groups["modifier"].Value : null;
            var parameters = m.Groups["params"].Success ? m.Groups["params"].Value : null;
            
            string? value = token switch
            {
                "Original" => fileName,
                "FileName" => fileName,
                "FolderName" => fileName,
                "JobName" => jobName,
                "Parent" => Path.GetFileName(dir) ?? "",
                "Date" => now.ToString("yyyy-MM-dd"),
                "Year" => now.ToString("yyyy"),
                "Ano" => now.ToString("yyyy"),
                "Month" => now.ToString("MM"),
                "Mes" => now.ToString("MM"),
                "Day" => now.ToString("dd"),
                "Dia" => now.ToString("dd"),
                "Time" => now.ToString("HH-mm-ss"),
                "DateTime" => now.ToString("yyyyMMdd_HHmmss"),
                "GUID" => Guid.NewGuid().ToString("N").Substring(0, 8),
                "Ext" => ext.TrimStart('.'),
                _ => null
            };

            if (value == null) return m.Value;
            return ApplyAdvancedModifier(value, modifier, parameters);
        });

        return result;
    }

    private string ApplyAdvancedModifier(string value, string? modifier, string? parameters)
    {
        if (string.IsNullOrEmpty(modifier)) return value;

        return modifier.ToLower() switch
        {
            "upper" => value.ToUpper(),
            "lower" => value.ToLower(),
            "title" => CultureInfo.CurrentCulture.TextInfo.ToTitleCase(value.ToLower()),
            "snake" => ToSnakeCase(value),
            "kebab" => ToKebabCase(value),
            "camel" => ToCamelCase(value),
            "pascal" => ToPascalCase(value),
            "regex" => ApplyRegex(value, parameters),
            "take" => parameters != null && int.TryParse(parameters, out var n) ? (value.Length > n ? value.Substring(0, n) : value) : value,
            "skip" => parameters != null && int.TryParse(parameters, out var s) ? (value.Length > s ? value.Substring(s) : "") : value,
            "trim" => value.Trim(),
            "clean" => Regex.Replace(value, @"[^a-zA-Z0-9\s\-_]+", ""),
            "no_spaces" => value.Replace(" ", "_"),
            "no_double_spaces" => Regex.Replace(value, @"\s+", " "),
            _ => value
        };
    }

    private string ApplyRegex(string value, string? parameters)
    {
        if (string.IsNullOrEmpty(parameters)) return value;

        // Tenta separar pattern e replacement. Ex: ( \d+ , ) ou ( \d+ )
        var parts = parameters.Split(',');
        var pattern = parts[0].Trim();
        var replacement = parts.Length > 1 ? parts[1].Trim() : "";

        try
        {
            return Regex.Replace(value, pattern, replacement);
        }
        catch
        {
            return value;
        }
    }

    private string GenerateUniqueDirectoryName(string path)
    {
        var parent = Path.GetDirectoryName(path) ?? "";
        var name = Path.GetFileName(path);
        int counter = 1;

        string newPath = path;
        while (Directory.Exists(newPath))
        {
            newPath = Path.Combine(parent, $"{name} ({counter})");
            counter++;
        }
        return newPath;
    }

    private string ToCamelCase(string text)
    {
        if (string.IsNullOrEmpty(text)) return text;
        var pascal = ToPascalCase(text);
        if (pascal.Length <= 1) return pascal.ToLower();
        return char.ToLower(pascal[0]) + pascal.Substring(1);
    }

    private string ToPascalCase(string text)
    {
        if (string.IsNullOrEmpty(text)) return text;
        var words = Regex.Split(text, @"[^a-zA-Z0-9]+").Where(w => !string.IsNullOrEmpty(w));
        return string.Concat(words.Select(w => char.ToUpper(w[0]) + w.Substring(1).ToLower()));
    }

    private string ToSnakeCase(string text)
    {
        if (string.IsNullOrEmpty(text)) return text;
        var result = Regex.Replace(text, @"([a-z0-9])([A-Z])", "$1_$2").ToLower();
        return Regex.Replace(result, @"[^a-z0-9]+", "_").Trim('_');
    }

    private string ToKebabCase(string text)
    {
        if (string.IsNullOrEmpty(text)) return text;
        var result = Regex.Replace(text, @"([a-z0-9])([A-Z])", "$1-$2").ToLower();
        return Regex.Replace(result, @"[^a-z0-9]+", "-").Trim('-');
    }

    private async Task TryRenameFileAsync(string template, string name, string filePath)
    {
        if (string.IsNullOrWhiteSpace(template)) return;

        var newPath = await GetRenamedPathAsync(template, name, filePath);
        
        if (newPath != filePath)
        {
            // Resoluo de Conflitos: Se j existir, adiciona (1), (2)...
            if (File.Exists(newPath))
            {
                newPath = GenerateUniqueFileName(newPath);
            }

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

    private string GenerateUniqueFileName(string path)
    {
        var dir = Path.GetDirectoryName(path) ?? "";
        var fileName = Path.GetFileNameWithoutExtension(path);
        var ext = Path.GetExtension(path);
        int counter = 1;

        string newPath = path;
        while (File.Exists(newPath))
        {
            newPath = Path.Combine(dir, $"{fileName} ({counter}){ext}");
            counter++;
        }
        return newPath;
    }
}
