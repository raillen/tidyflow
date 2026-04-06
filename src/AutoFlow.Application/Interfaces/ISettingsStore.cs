using System.Threading.Tasks;
using AutoFlow.Domain.Entities;

namespace AutoFlow.Application.Interfaces;

public interface ISettingsStore
{
    Task<AppSettings> LoadAsync();
    AppSettings Load();
    Task SaveAsync(AppSettings settings);
}
