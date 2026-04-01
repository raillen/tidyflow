using System;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using FolderFlow.Application.Interfaces;
using FolderFlow.Application.Services;
using FolderFlow.Domain.Entities;
using FolderFlow.Domain.ValueObjects;
using System.Diagnostics;
using System.IO;

namespace FolderFlow.App.ViewModels;

public partial class JobItemViewModel : ViewModelBase
{
    private readonly JobAppService _jobAppService;
    private readonly INotificationService _notificationService;

    [ObservableProperty] private bool _isSelected;
    [ObservableProperty] private bool _isExpanded;
    [ObservableProperty] private string _status = "Ocioso";
    [ObservableProperty] private double _progress;
    [ObservableProperty] private string _currentFile = string.Empty;
    [ObservableProperty] private string _speedText = "0 KB/s";
    [ObservableProperty] private ObservableCollection<string> _recentFilesLog = new();

    public Job Job { get; }

    public JobItemViewModel(Job job, JobAppService jobAppService, INotificationService notificationService)
    {
        Job = job;
        _jobAppService = jobAppService;
        _notificationService = notificationService;
    }

    partial void OnIsSelectedChanged(bool value)
    {
        var mainVm = App.Services?.GetService(typeof(MainWindowViewModel)) as MainWindowViewModel;
        mainVm?.Automation.UpdateSelectionState();
    }

    public void UpdateFromProgress(JobProgressInfo p)
    {
        Status = p.Status;
        Progress = p.TotalPercentage;
        CurrentFile = p.CurrentFile;
        SpeedText = FormatSpeed(p.TransferSpeed);
        
        if (p.RecentFilesLog.Any())
        {
            Dispatcher.UIThread.Post(() => {
                // Atualizao inteligente: s altera se o log for novo
                if (RecentFilesLog.Count == 0 || RecentFilesLog[0] != p.RecentFilesLog[0])
                {
                    RecentFilesLog.Clear();
                    foreach (var log in p.RecentFilesLog) RecentFilesLog.Add(log);
                }
            });
        }
    }

    private string FormatSpeed(double bytesPerSecond)
    {
        if (bytesPerSecond < 1024) return $"{bytesPerSecond:F0} B/s";
        if (bytesPerSecond < 1024 * 1024) return $"{bytesPerSecond / 1024:F1} KB/s";
        return $"{bytesPerSecond / (1024 * 1024):F1} MB/s";
    }

    [RelayCommand]
    public void ToggleExpand() => IsExpanded = !IsExpanded;

    [RelayCommand]
    public async Task RunNow()
    {
        Status = "Na Fila...";
        await _jobAppService.RunJobAsync(Job.Id);
    }

    [RelayCommand]
    public void Edit()
    {
        (App.Services?.GetService(typeof(MainWindowViewModel)) as MainWindowViewModel)?.NavigateToJobs("Edit");
    }

    [RelayCommand]
    private void OpenSource() => OpenPath(Job.SourcePath);

    [RelayCommand]
    private void OpenTarget() => OpenPath(Job.TargetPath);

    private void OpenPath(string path)
    {
        try { if (Directory.Exists(path)) Process.Start("explorer.exe", path); } catch { }
    }
}
