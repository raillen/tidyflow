using System.Threading.Tasks;

namespace AutoFlow.Application.Interfaces;

public interface IStorageService
{
    Task<string?> SelectFolderAsync();
    Task<string?> SaveFileAsync(string defaultName, string extensionName, string extension);
    Task<string?> OpenFileAsync(string extensionName, string extension);
}
