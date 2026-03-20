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
    private void NavigateToDashboard() => CurrentPage = Dashboard;

    [RelayCommand]
    private void NavigateToJobs() => CurrentPage = Jobs;

    [RelayCommand]
    private void NavigateToHistory() => CurrentPage = History;

    [RelayCommand]
    private void NavigateToSettings() => CurrentPage = Settings;
}
