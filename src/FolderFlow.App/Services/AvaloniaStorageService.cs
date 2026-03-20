using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Platform.Storage;
using FolderFlow.Application.Interfaces;

namespace FolderFlow.App.Services;

public class AvaloniaStorageService : IStorageService
{
    private Window? GetMainWindow()
    {
        if (Avalonia.Application.Current?.ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
        {
            return desktop.MainWindow;
        }
        return null;
    }

    public async Task<string?> SelectFolderAsync()
    {
        var window = GetMainWindow();
        if (window == null) return null;

        var folders = await window.StorageProvider.OpenFolderPickerAsync(new FolderPickerOpenOptions
        {
            Title = "Selecionar Pasta",
            AllowMultiple = false
        });

        return folders.FirstOrDefault()?.Path.LocalPath;
    }

    public async Task<string?> SaveFileAsync(string defaultName, string extensionName, string extension)
    {
        var window = GetMainWindow();
        if (window == null) return null;

        var file = await window.StorageProvider.SaveFilePickerAsync(new FilePickerSaveOptions
        {
            Title = "Salvar Exportação",
            SuggestedFileName = defaultName,
            DefaultExtension = extension,
            FileTypeChoices = new[]
            {
                new FilePickerFileType(extensionName) { Patterns = new[] { $"*.{extension}" } }
            }
        });

        return file?.Path.LocalPath;
    }

    public async Task<string?> OpenFileAsync(string extensionName, string extension)
    {
        var window = GetMainWindow();
        if (window == null) return null;

        var files = await window.StorageProvider.OpenFilePickerAsync(new FilePickerOpenOptions
        {
            Title = "Abrir Arquivo",
            AllowMultiple = false,
            FileTypeFilter = new[]
            {
                new FilePickerFileType(extensionName) { Patterns = new[] { $"*.{extension}" } }
            }
        });

        return files.FirstOrDefault()?.Path.LocalPath;
    }
}
