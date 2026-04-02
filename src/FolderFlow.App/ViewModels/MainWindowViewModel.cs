using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using FolderFlow.Domain.Entities;
using FolderFlow.Application.Interfaces;
using System;
using System.Threading.Tasks;
using Microsoft.Extensions.DependencyInjection;

namespace FolderFlow.App.ViewModels;

public partial class MainWindowViewModel : ViewModelBase
{
    private readonly ILocalizationService _localizationService;
    private readonly IServiceProvider _serviceProvider;

    [ObservableProperty]
    private ViewModelBase _currentPage;

    private DashboardViewModel? _dashboard;
    public DashboardViewModel Dashboard => _dashboard ??= _serviceProvider.GetRequiredService<DashboardViewModel>();

    private AutomationViewModel? _automation;
    public AutomationViewModel Automation => _automation ??= _serviceProvider.GetRequiredService<AutomationViewModel>();

    private HistoryViewModel? _history;
    public HistoryViewModel History => _history ??= _serviceProvider.GetRequiredService<HistoryViewModel>();

    private SettingsViewModel? _settings;
    public SettingsViewModel Settings => _settings ??= _serviceProvider.GetRequiredService<SettingsViewModel>();

    private BlueprintViewModel? _blueprint;
    public BlueprintViewModel Blueprint => _blueprint ??= _serviceProvider.GetRequiredService<BlueprintViewModel>();

    public MainWindowViewModel(
        IServiceProvider serviceProvider,
        ILocalizationService localizationService)
    {
        _serviceProvider = serviceProvider;
        _localizationService = localizationService;
        
        // Inicializa a primeira pgina - Removido para evitar loop no DI durante construo
        // _currentPage = Dashboard;
    }

    [RelayCommand] public void NavigateToDashboard() => CurrentPage = Dashboard;
    [RelayCommand] public void NavigateToAutomation() => CurrentPage = Automation;
    [RelayCommand] public void NavigateToHistory() => CurrentPage = History;
    [RelayCommand] public void NavigateToSettings() => CurrentPage = Settings;
    [RelayCommand] public void NavigateToBlueprint() => CurrentPage = Blueprint;

    [RelayCommand]
    public async Task ShowDonate()
    {
        var window = new Views.DonateWindow
        {
            DataContext = new DonateViewModel(_localizationService)
        };

        if (Avalonia.Application.Current?.ApplicationLifetime is Avalonia.Controls.ApplicationLifetimes.IClassicDesktopStyleApplicationLifetime desktop)
        {
            await window.ShowDialog(desktop.MainWindow!);
        }
    }

    public void ShowEditor(Job job)
    {
        var window = new Views.JobEditorWindow();
        var vm = _serviceProvider.GetRequiredService<JobEditorViewModel>();
        
        _ = vm.SetJob(job);
        window.DataContext = vm;
        
        vm.Saved += () => {
            window.Close();
            _ = Automation.LoadJobsAsync();
        };
        vm.Cancelled += () => window.Close();

        if (Avalonia.Application.Current?.ApplicationLifetime is Avalonia.Controls.ApplicationLifetimes.IClassicDesktopStyleApplicationLifetime desktop)
        {
            _ = window.ShowDialog(desktop.MainWindow!);
        }
    }

    public void ShowBlueprintEditor(Blueprint blueprint)
    {
        var window = new Views.BlueprintEditorWindow();
        var vm = _serviceProvider.GetRequiredService<BlueprintEditorViewModel>();
        
        vm.SetBlueprint(blueprint);
        window.DataContext = vm;
        
        vm.Saved += () => {
            window.Close();
            _ = Blueprint.LoadBlueprintsAsync();
        };
        vm.Cancelled += () => window.Close();

        if (Avalonia.Application.Current?.ApplicationLifetime is Avalonia.Controls.ApplicationLifetimes.IClassicDesktopStyleApplicationLifetime desktop)
        {
            _ = window.ShowDialog(desktop.MainWindow!);
        }
    }

    public void NavigateToPage(string pageName)
    {
        switch (pageName)
        {
            case "Dashboard": NavigateToDashboard(); break;
            case "Automation": NavigateToAutomation(); break;
            case "History": NavigateToHistory(); break;
            case "Settings": NavigateToSettings(); break;
            case "Blueprint": NavigateToBlueprint(); break;
        }
    }

    public void NavigateToJobs(string mode)
    {
        if (mode == "DirectCopy") 
        {
            ShowEditor(new Job { WatchEnabled = false, Name = _localizationService["NewDirectCopy"] });
        }
        else if (mode == "WatchFolder")
        {
            ShowEditor(new Job { WatchEnabled = true, Name = _localizationService["NewWatchFolder"] });
        }
        else
        {
            CurrentPage = Automation;
        }
    }
}
