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
    private readonly ILocalizationService _localizationService;

    [ObservableProperty]
    private AppSettings _settings = new();

    public double GlassOpacity
    {
        get => Settings.GlassOpacity;
        set
        {
            if (Settings.GlassOpacity != value)
            {
                Settings.GlassOpacity = value;
                OnPropertyChanged(nameof(GlassOpacity));
                // Forçamos a atualização do binding na MainWindow também
                OnPropertyChanged(nameof(Settings));
            }
        }
    }

    public ObservableCollection<ThemeMode> Themes { get; } = new(new[] { 
        ThemeMode.System, 
        ThemeMode.Light, 
        ThemeMode.Dark,
        ThemeMode.Dracula,
        ThemeMode.Neon
    });
    
    public ObservableCollection<string> Languages { get; } = new(new[] { 
        "pt-BR", "en-US", "es-ES", "ja-JP", "ru-RU" 
    });

    public SettingsViewModel(
        ISettingsStore settingsStore, 
        ThemeService themeService,
        ILocalizationService localizationService)
    {
        _settingsStore = settingsStore;
        _themeService = themeService;
        _localizationService = localizationService;
        
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
        
        // Aplica o tema imediatamente
        _themeService.SetTheme(Settings.Theme);
        
        // Aplica o idioma imediatamente
        if (!string.IsNullOrEmpty(Settings.Language))
        {
            _localizationService.SetLanguage(Settings.Language);
        }
    }
}
