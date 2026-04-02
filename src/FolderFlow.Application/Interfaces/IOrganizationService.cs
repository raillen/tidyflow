using System.Threading.Tasks;
using FolderFlow.Domain.Entities;

namespace FolderFlow.Application.Interfaces;

public interface IOrganizationService
{
    Task ProcessBlueprintAsync(Blueprint blueprint);
    Task ApplyScaffoldingAsync(Blueprint blueprint, string newFolderPath);
    Task<string> GetRenamedPathAsync(string renameTemplate, string jobName, string originalPath);
}
