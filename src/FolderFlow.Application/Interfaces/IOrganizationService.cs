using System.Threading.Tasks;
using FolderFlow.Domain.Entities;

namespace FolderFlow.Application.Interfaces;

public interface IOrganizationService
{
    Task ProcessOrganizationAsync(Job job);
    Task ApplyScaffoldingAsync(Job job, string newFolderPath);
    Task<string> GetRenamedPathAsync(Job job, string originalPath);
}
