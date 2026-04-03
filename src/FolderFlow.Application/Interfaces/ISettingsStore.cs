using System.Threading.Tasks;
using FolderFlow.Domain.Entities;

namespace FolderFlow.Application.Interfaces;

public interface ISettingsStore
{
    Task<AppSettings> LoadAsync();
    AppSettings Load();
    Task SaveAsync(AppSettings settings);
}
