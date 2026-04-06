using System;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using AutoFlow.Application.Interfaces;
using AutoFlow.Application.Services;
using AutoFlow.Domain.Entities;
using AutoFlow.Domain.ValueObjects;
using System.Diagnostics;
using System.IO;

namespace AutoFlow.App.ViewModels;

public partial class JobItemViewModel : ViewModelBase
{
    private readonly JobAppService _jobAppService;
    private readonly INotificationService _notificationService;
    private readonly ILocalizationService _localizationService;

    [ObservableProperty] private bool _isSelected;
    [ObservableProperty] private bool _isExpanded;
    [ObservableProperty] private bool _isSimulating;
    [ObservableProperty] 
    [NotifyPropertyChangedFor(nameof(LocalizedStatus))]
    private string _status = "";
    [ObservableProperty] private double _progress;

    public string LocalizedStatus
    {
        get
        {
            if (string.IsNullOrEmpty(Status)) return "";
            
            var upper = Status.ToUpper();
            if (upper == "OCIOSO") return _localizationService["Idle"];
            if (upper == "PROCESSANDO...") return _localizationService["Processing"];
            if (upper == "NA FILA...") return _localizationService["Queued"];
            if (upper == "HIDRATANDO") return _localizationService["Downloading"];
            if (upper == "COPIADO") return _localizationService["Copied"];
            if (upper == "MOVIDO") return _localizationService["Moved"];
            if (upper == "IGNORADO") return _localizationService["Ignored"];
            if (upper == "FALHA") return _localizationService["FailedStatus"];
            
            return Status;
        }
    }

    [ObservableProperty] private string _currentFile = string.Empty;
    [ObservableProperty] private string _speedText = "0 KB/s";
    [ObservableProperty] private ObservableCollection<string> _recentFilesLog = new();

    public Job Job { get; }

    public string ShortSource => SummarizePath(Job.SourcePath);
    public string ShortTarget => SummarizePath(Job.TargetPath);

    private string SummarizePath(string path)
    {
        if (string.IsNullOrEmpty(path)) return string.Empty;
        if (path.Length <= 45) return path;
        return path.Substring(0, 15) + "..." + path.Substring(path.Length - 25);
    }

    public JobItemViewModel(Job job, JobAppService jobAppService, INotificationService notificationService, ILocalizationService localizationService)
    {
        Job = job;
        _jobAppService = jobAppService;
        _notificationService = notificationService;
        _localizationService = localizationService;
        _status = _localizationService["Idle"];
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
        Status = _localizationService["Queued"];
        await _jobAppService.RunJobAsync(Job.Id);
    }

    [RelayCommand]
    public async Task SimulateAsync()
    {
        if (IsSimulating) return;
        IsSimulating = true;
        try
        {
            var previewEngine = App.Services?.GetService(typeof(PreviewEngine)) as PreviewEngine;
            if (previewEngine != null)
            {
                var summary = await previewEngine.GeneratePreviewAsync(Job);
                
                Dispatcher.UIThread.Post(() => {
                    var vm = new PreviewWindowViewModel(summary, _localizationService);
                    var window = new Views.PreviewWindow { DataContext = vm };
                    
                    var desktop = Avalonia.Application.Current?.ApplicationLifetime as Avalonia.Controls.ApplicationLifetimes.IClassicDesktopStyleApplicationLifetime;
                    var mainWindow = desktop?.MainWindow;
                    if (mainWindow != null)
                    {
                        window.ShowDialog(mainWindow);
                    }
                    else
                    {
                        window.Show();
                    }
                });
            }
        }
        finally
        {
            IsSimulating = false;
        }
    }

    [RelayCommand]
    public void Edit()
    {
        var mainVm = App.Services?.GetService(typeof(MainWindowViewModel)) as MainWindowViewModel;
        mainVm?.ShowEditor(Job);
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
