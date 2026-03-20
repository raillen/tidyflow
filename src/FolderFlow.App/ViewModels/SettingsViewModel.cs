using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using FolderFlow.App.Services;
using FolderFlow.Application.Interfaces;
using FolderFlow.Domain.Entities;
using FolderFlow.Domain.Enums;

namespace FolderFlow.App.ViewModels;

public partial class SettingsViewModel : ViewModelBase
{
    private readonly ISettingsStore _settingsStore;
    private readonly ThemeService _themeService;

    [ObservableProperty]
    private AppSettings _settings = new();

    public ObservableCollection<ThemeMode> Themes { get; } = new(new[] { ThemeMode.System, ThemeMode.Light, ThemeMode.Dark });
    public ObservableCollection<string> Languages { get; } = new(new[] { "pt-BR", "en-US" });

    public SettingsViewModel(ISettingsStore settingsStore, ThemeService themeService)
    {
        _settingsStore = settingsStore;
        _themeService = themeService;
        
        LoadSettingsCommand.Execute(null);
    }

    [RelayCommand]
    private async Task LoadSettings()
    {
        Settings = await _settingsStore.LoadAsync();
    }

    [RelayCommand]
    private async Task SaveSettings()
    {
        await _settingsStore.SaveAsync(Settings);
        _themeService.SetTheme(Settings.Theme);
    }
}
