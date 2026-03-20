using System.Threading.Tasks;
using FolderFlow.Domain.Entities;

namespace FolderFlow.Application.Interfaces;

public interface ISettingsStore
{
    Task<AppSettings> LoadAsync();
    Task SaveAsync(AppSettings settings);
}
