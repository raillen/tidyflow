using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;

namespace FolderFlow.App.ViewModels;

public partial class MainWindowViewModel : ViewModelBase
{
    [ObservableProperty]
    private ViewModelBase _currentPage;

    public DashboardViewModel Dashboard { get; }
    public JobsViewModel Jobs { get; }
    public HistoryViewModel History { get; }
    public SettingsViewModel Settings { get; }

    public MainWindowViewModel(
        DashboardViewModel dashboard, 
        JobsViewModel jobs, 
        HistoryViewModel history,
        SettingsViewModel settings)
    {
        Dashboard = dashboard;
        Jobs = jobs;
        History = history;
        Settings = settings;
        _currentPage = Dashboard;
    }

    [RelayCommand]
    public void NavigateToDashboard() => CurrentPage = Dashboard;

    [RelayCommand]
    public void NavigateToJobs() => CurrentPage = Jobs;

    [RelayCommand]
    public void NavigateToHistory() => CurrentPage = History;

    [RelayCommand]
    public void NavigateToSettings() => CurrentPage = Settings;

    public void NavigateToPage(string pageName)
    {
        switch (pageName)
        {
            case "Dashboard": NavigateToDashboard(); break;
            case "Jobs": NavigateToJobs(); break;
            case "History": NavigateToHistory(); break;
            case "Settings": NavigateToSettings(); break;
        }
    }

    public void NavigateToJobs(string mode)
    {
        CurrentPage = Jobs;
        if (mode == "DirectCopy") Jobs.CreateDirectCopyJobCommand.Execute(null);
        else if (mode == "WatchFolder") Jobs.CreateWatchFolderJobCommand.Execute(null);
    }
}
