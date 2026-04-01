using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using FolderFlow.Domain.Entities;
using System;

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
        // Esta verso evita o loop infinito ao verificar se j estamos na Automation
        if (mode == "DirectCopy") 
        {
            ShowEditor(new Job { WatchEnabled = false, Name = "Nova Cpia Direta" });
        }
        else if (mode == "WatchFolder")
        {
            ShowEditor(new Job { WatchEnabled = true, Name = "Nova Watch Folder" });
        }
        else if (mode == "Edit")
        {
            // O Edit  geralmente chamado via JobItemViewModel, que agora chamar ShowEditor diretamente
        }
        else
        {
            CurrentPage = Automation;
        }
    }

    public async void ShowEditor(Job job)
    {
        var editor = App.Services?.GetService(typeof(JobEditorViewModel)) as JobEditorViewModel;
        if (editor != null)
        {
            await editor.SetJob(job);
            
            var window = new FolderFlow.App.Views.JobEditorWindow { DataContext = editor };
            
            // Eventos para fechar a janela
            editor.Saved += () => { 
                window.Close();
                _ = Automation.LoadJobsAsync(); 
            };
            editor.Cancelled += () => { 
                window.Close(); 
            };

            // Abre como modal sobre a MainWindow
            if (Avalonia.Application.Current?.ApplicationLifetime is Avalonia.Controls.ApplicationLifetimes.IClassicDesktopStyleApplicationLifetime desktop)
            {
                await window.ShowDialog(desktop.MainWindow!);
            }
        }
    }
}
