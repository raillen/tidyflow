using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using AutoFlow.App.Services;
using AutoFlow.Application.Interfaces;
using AutoFlow.Domain.Entities;
using AutoFlow.Domain.Enums;
using AutoFlow.Infrastructure.Logging;

namespace AutoFlow.App.ViewModels;

public partial class SettingsViewModel : ViewModelBase
{
    private readonly ISettingsStore _settingsStore;
    private readonly ThemeService _themeService;
    private readonly ILocalizationService _localizationService;

    [ObservableProperty]
    private AppSettings _settings = new();

    public ThemeMode SelectedTheme
    {
        get => Settings.Theme;
        set
        {
            if (Settings.Theme != value)
            {
                Settings.Theme = value;
                OnPropertyChanged(nameof(SelectedTheme));
                _themeService.SetTheme(value);
            }
        }
    }

    public AppFont SelectedFont
    {
        get => Settings.Font;
        set
        {
            if (Settings.Font != value)
            {
                Settings.Font = value;
                OnPropertyChanged(nameof(SelectedFont));
                _themeService.SetFont(value);
            }
        }
    }

    public string SelectedGlassMaterial
    {
        get => Settings.GlassMaterial;
        set
        {
            if (Settings.GlassMaterial != value)
            {
                Settings.GlassMaterial = value;
                OnPropertyChanged(nameof(SelectedGlassMaterial));
                // Notifica a MainWindow que o objeto Settings mudou para atualizar o TransparencyLevelHint
                OnPropertyChanged(nameof(Settings));
            }
        }
    }

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

    public ObservableCollection<AppFont> Fonts { get; } = new(new[] {
        AppFont.Default,
        AppFont.System,
        AppFont.Roboto,
        AppFont.Montserrat,
        AppFont.OpenSans,
        AppFont.PublicSans,
        AppFont.JetBrains
    });

    public ObservableCollection<string> GlassMaterials { get; } = new(new[] {
        "None",
        "Mica",
        "AcrylicBlur"
    });
    
    public ObservableCollection<string> Languages { get; } = new(new[] { 
        "pt-BR", "en-US", "es-ES", "ja-JP", "ru-RU" 
    });

    public ObservableCollection<string> WebhookTypes { get; } = new(new[] { "Generic", "Discord", "Slack" });

    private readonly IAuditService _auditService;
    private readonly INotificationService _notificationService;

    [ObservableProperty] private string _databaseSize = "0 KB";
    [ObservableProperty] private string _databaseSizeText = string.Empty;
    public ObservableCollection<int> RetentionOptions { get; } = new(new[] { 0, 7, 15, 30, 60, 90 });

    public SettingsViewModel(
        ISettingsStore settingsStore, 
        ThemeService themeService,
        ILocalizationService localizationService,
        IAuditService auditService,
        INotificationService notificationService)
    {
        _settingsStore = settingsStore;
        _themeService = themeService;
        _localizationService = localizationService;
        _auditService = auditService;
        _notificationService = notificationService;
        
        if (_localizationService is System.ComponentModel.INotifyPropertyChanged npc)
        {
            npc.PropertyChanged += (s, e) => {
                if (e.PropertyName == "Item" || e.PropertyName == "Item[]" || string.IsNullOrEmpty(e.PropertyName))
                {
                    RefreshDatabaseStats();
                }
            };
        }

        LoadSettingsCommand.Execute(null);
        RefreshDatabaseStats();
    }

    private void RefreshDatabaseStats()
    {
        if (_auditService is SqliteAuditService sqlite)
        {
            var bytes = sqlite.GetDatabaseSize();
            if (bytes < 1024) DatabaseSize = $"{bytes} B";
            else if (bytes < 1024 * 1024) DatabaseSize = $"{bytes / 1024.0:F1} KB";
            else DatabaseSize = $"{bytes / (1024.0 * 1024.0):F1} MB";

            DatabaseSizeText = string.Format(_localizationService["DatabaseSize"], DatabaseSize);
        }
    }

    [RelayCommand]
    private async Task OptimizeDatabase()
    {
        if (_auditService is SqliteAuditService sqlite)
        {
            await sqlite.VacuumAsync();
            RefreshDatabaseStats();
        }
    }

    [RelayCommand]
    private async Task PurgeOldLogs()
    {
        if (_auditService is SqliteAuditService sqlite)
        {
            await sqlite.PurgeOldLogsAsync(Settings.LogRetentionDays);
            RefreshDatabaseStats();
        }
    }

    [RelayCommand]
    private async Task LoadSettings()
    {
        Settings = await _settingsStore.LoadAsync();
    }

    public ObservableCollection<string> Priorities { get; } = new(new[] { "BelowNormal", "Normal", "High" });
    public ObservableCollection<int> Threads { get; } = new(new[] { 1, 2, 3, 4, 5, 8, 10 });

    [RelayCommand]
    private void OpenLink(string url)
    {
        try
        {
            System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo
            {
                FileName = url,
                UseShellExecute = true
            });
        }
        catch { }
    }

    [RelayCommand]
    private void ShowChangelog()
    {
        var window = new AutoFlow.App.Views.ChangelogWindow();
        
        // Tenta pegar a janela principal para centralizar o modal
        if (Avalonia.Application.Current?.ApplicationLifetime is Avalonia.Controls.ApplicationLifetimes.IClassicDesktopStyleApplicationLifetime desktop)
        {
            window.ShowDialog(desktop.MainWindow!);
        }
        else
        {
            window.Show();
        }
    }

    [RelayCommand]
    private async Task SaveSettings()
    {
        await _settingsStore.SaveAsync(Settings);
        
        // Aplica o tema imediatamente
        _themeService.SetTheme(Settings.Theme);
        _themeService.SetFont(Settings.Font);
        
        // Aplica o idioma imediatamente
        if (!string.IsNullOrEmpty(Settings.Language))
        {
            _localizationService.SetLanguage(Settings.Language);
        }

        // Integrao Windows
        if (OperatingSystem.IsWindows())
        {
            AutoFlow.Infrastructure.Helpers.WindowsStartupHelper.SetStartup(Settings.StartAtStartup);
        }
        AutoFlow.Infrastructure.Helpers.WindowsStartupHelper.SetProcessPriority(Settings.ProcessPriority);
    }
}
