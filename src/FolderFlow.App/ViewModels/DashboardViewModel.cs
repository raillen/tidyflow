using System;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using FolderFlow.Application.Services;
using System.Collections.ObjectModel;
using System.Collections.Generic;
using System.IO;

namespace FolderFlow.App.ViewModels;

public partial class DashboardViewModel : ViewModelBase
{
    private readonly JobAppService _jobAppService;
    private readonly QueueProcessor _queueProcessor;
    private readonly DispatcherTimer _timer;
    private readonly string _reportsFolder;

    [ObservableProperty]
    private int _totalJobs;

    [ObservableProperty]
    private int _activeWatchers;

    [ObservableProperty]
    private int _currentlyRunning;

    [ObservableProperty]
    private int _totalFilesProcessed;

    [ObservableProperty]
    private int _totalErrors;

    [ObservableProperty]
    private ObservableCollection<string> _recentActivity = new();

    public DashboardViewModel(JobAppService jobAppService, QueueProcessor queueProcessor)
    {
        _jobAppService = jobAppService;
        _queueProcessor = queueProcessor;
        
        var dataFolder = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Data");
        _reportsFolder = Path.Combine(dataFolder, "Reports");

        LoadStatsCommand.Execute(null);

        _timer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(5) };
        _timer.Tick += (s, e) => {
            UpdateRealtimeStats();
            _ = LoadStats();
        };
        _timer.Start();
    }

    [RelayCommand]
    private async Task LoadStats()
    {
        var jobs = await _jobAppService.GetAllJobsAsync();
        var jobsList = jobs.ToList();
        TotalJobs = jobsList.Count;
        ActiveWatchers = jobsList.Count(j => j.WatchEnabled);
        
        await CalculateAuditStats();
        UpdateRealtimeStats();
    }

    private async Task CalculateAuditStats()
    {
        if (!Directory.Exists(_reportsFolder)) return;

        int processed = 0;
        int errors = 0;
        var activity = new List<string>();

        try
        {
            var files = Directory.GetFiles(_reportsFolder, "*.csv").OrderByDescending(f => File.GetCreationTime(f)).Take(10);

            foreach (var file in files)
            {
                var lines = await File.ReadAllLinesAsync(file);
                for (int i = 1; i < lines.Length; i++)
                {
                    var parts = lines[i].Split(';');
                    if (parts.Length >= 3)
                    {
                        var status = parts[2].Trim('"');
                        if (status == "COPIADO" || status == "MOVIDO") processed++;
                        else if (status.Contains("FALHA")) errors++;

                        if (activity.Count < 10)
                        {
                            activity.Add($"{parts[0]} - {parts[1].Trim('\"')}: {status}");
                        }
                    }
                }
            }

            TotalFilesProcessed = processed;
            TotalErrors = errors;
            
            Avalonia.Threading.Dispatcher.UIThread.Post(() => {
                RecentActivity.Clear();
                foreach (var item in activity) RecentActivity.Add(item);
            });
        }
        catch { }
    }

    private void UpdateRealtimeStats()
    {
        CurrentlyRunning = _queueProcessor.ActiveCount;
    }
}
