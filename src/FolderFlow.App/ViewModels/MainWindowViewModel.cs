using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using FolderFlow.Domain.Entities;
using FolderFlow.Application.Interfaces;
using System;
using System.Threading.Tasks;

namespace FolderFlow.App.ViewModels;

public partial class MainWindowViewModel : ViewModelBase
{
    private readonly ILocalizationService _localizationService;

    [ObservableProperty]
    private ViewModelBase _currentPage;

    public DashboardViewModel Dashboard { get; }
    public AutomationViewModel Automation { get; }
    public HistoryViewModel History { get; }
    public SettingsViewModel Settings { get; }
    public BlueprintViewModel Blueprint { get; }

    public MainWindowViewModel(
        DashboardViewModel dashboard, 
        AutomationViewModel automation, 
        HistoryViewModel history,
        SettingsViewModel settings,
        BlueprintViewModel blueprint,
        ILocalizationService localizationService)
    {
        Dashboard = dashboard;
        Automation = automation;
        History = history;
        Settings = settings;
        Blueprint = blueprint;
        _localizationService = localizationService;
        _currentPage = Dashboard;
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
        var vm = App.Services?.GetService(typeof(JobEditorViewModel)) as JobEditorViewModel;
        if (vm != null)
        {
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
    }

    public void ShowBlueprintEditor(Blueprint blueprint)
    {
        var window = new Views.BlueprintEditorWindow();
        var vm = App.Services?.GetService(typeof(BlueprintEditorViewModel)) as BlueprintEditorViewModel;
        if (vm != null)
        {
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
