using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;

namespace FolderFlow.App.ViewModels;

public partial class MainWindowViewModel : ViewModelBase
{
    [ObservableProperty]
    private ViewModelBase _currentPage;

    public DashboardViewModel Dashboard { get; }
    public AutomationViewModel Automation { get; }
    public HistoryViewModel History { get; }
    public SettingsViewModel Settings { get; }

    public MainWindowViewModel(
        DashboardViewModel dashboard, 
        AutomationViewModel automation, 
        HistoryViewModel history,
        SettingsViewModel settings)
    {
        Dashboard = dashboard;
        Automation = automation;
        History = history;
        Settings = settings;
        _currentPage = Dashboard;
    }

    [RelayCommand]
    public void NavigateToDashboard() => CurrentPage = Dashboard;

    [RelayCommand]
    public void NavigateToAutomation() => CurrentPage = Automation;

    [RelayCommand]
    public void NavigateToHistory() => CurrentPage = History;

    [RelayCommand]
    public void NavigateToSettings() => CurrentPage = Settings;

    public void NavigateToPage(string pageName)
    {
        switch (pageName)
        {
            case "Dashboard": NavigateToDashboard(); break;
            case "Automation": NavigateToAutomation(); break;
            case "History": NavigateToHistory(); break;
            case "Settings": NavigateToSettings(); break;
        }
    }

    public void NavigateToJobs(string mode)
    {
        CurrentPage = Automation;
        if (mode == "DirectCopy") Automation.CreateDirectCopyCommand.Execute(null);
        else if (mode == "WatchFolder") Automation.CreateWatchFolderCommand.Execute(null);
    }
}
