using System.Threading.Tasks;
using FolderFlow.Domain.Entities;

namespace FolderFlow.Application.Interfaces;

public interface IOrganizationService
{
    Task ProcessBlueprintAsync(Blueprint blueprint, string? eventPath = null);
    Task ApplyScaffoldingAsync(Blueprint blueprint, string newFolderPath);
    Task<string> GetRenamedPathAsync(string renameTemplate, string jobName, string originalPath);
    Task<string> GetRenamedPathAsync(string renameTemplate, string jobName, string originalPath, bool isFolder);
    Task<string> GetRenamedPathAsync(string renameTemplate, string jobName, string originalPath, bool isFolder, int counterStart);
}
