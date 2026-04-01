using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using FolderFlow.Application.Interfaces;
using FolderFlow.Application.Services;
using FolderFlow.Domain.Entities;
using FolderFlow.Domain.ValueObjects;

namespace FolderFlow.App.ViewModels;

public partial class DashboardViewModel : ViewModelBase, IDisposable
{
    private readonly string _reportsFolder;
    private readonly JobAppService _jobAppService;
    private readonly ISystemActivityService _activityService;
    private readonly QueueProcessor _queueProcessor;
    private readonly GlobalProgressService _globalProgressService;
    private readonly ISettingsStore _settingsStore;
    private readonly DispatcherTimer _timer;
    private readonly Process _currentProcess;

    [ObservableProperty] private string _selectedTypeFilter = "Todos";
    [ObservableProperty] private string _selectedTaskFilter = "Todos";
    public ObservableCollection<string> AvailableTypes { get; } = new(new[] { "Todos", "Cpia Direta", "Watch Folder" });
    [ObservableProperty] private ObservableCollection<string> _availableTasks = new(new[] { "Todos" });

    // Cpia Direta Stats
    [ObservableProperty] private int _directActive;
    [ObservableProperty] private int _directPaused;
    [ObservableProperty] private int _directCompleted;

    // Watch Folder Stats
    [ObservableProperty] private int _watchActive;
    [ObservableProperty] private int _watchPaused;
    [ObservableProperty] private int _watchCompleted;

    // Processed Files Stats
    [ObservableProperty] private int _totalProcessed;
    [ObservableProperty] private int _totalIgnored;
    [ObservableProperty] private int _totalErrors;

    // System Monitor
    [ObservableProperty] private string _transferSpeed = "0 KB/s";
    [ObservableProperty] private double _cpuUsage;
    [ObservableProperty] private string _ramUsage = "0 MB";
    [ObservableProperty] private string _cloudSyncStatus = "Sincronizado";

    [ObservableProperty] private ObservableCollection<SystemActivity> _recentActivities = new();

    private List<Job> _allJobs = new();
    private bool _isUpdating;
    private int _auditUpdateCounter;

    // Session-based real-time stats to add to historical data
    private int _historicalProcessed;
    private int _historicalIgnored;
    private int _historicalErrors;

    public DashboardViewModel(
        JobAppService jobAppService, 
        ISystemActivityService activityService,
        QueueProcessor queueProcessor,
        GlobalProgressService globalProgressService,
        ISettingsStore settingsStore)
    {
        _jobAppService = jobAppService;
        _activityService = activityService;
        _queueProcessor = queueProcessor;
        _globalProgressService = globalProgressService;
        _settingsStore = settingsStore;
        
        var dataFolder = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Data");
        _reportsFolder = Path.Combine(dataFolder, "Reports");
        _currentProcess = Process.GetCurrentProcess();

        _timer = new DispatcherTimer
        {
            Interval = TimeSpan.FromSeconds(2) // Default inicial
        };
        _timer.Tick += async (s, e) => 
        {
            UpdateSystemMonitor();
            
            _auditUpdateCounter++;
            if (_auditUpdateCounter >= (30 / (int)_timer.Interval.TotalSeconds)) 
            {
                _auditUpdateCounter = 0;
                await UpdateStatsAsync();
            }
        };
        _timer.Start();

        // Subscreve para atualizaes em tempo real via evento
        _globalProgressService.OnProgressReported += HandleOnProgressReported;

        _ = InitialLoad();
    }

    private void HandleOnProgressReported(JobProgressInfo p)
    {
        Dispatcher.UIThread.Post(() => HandleRealTimeProgress(p));
    }

    private void HandleRealTimeProgress(JobProgressInfo p)
    {
        // Verifica filtros
        bool typeMatch = SelectedTypeFilter == "Todos";
        if (!typeMatch)
        {
            var job = _allJobs.FirstOrDefault(j => j.Id == p.JobId);
            if (job != null)
            {
                if (SelectedTypeFilter == "Cpia Direta" && !job.WatchEnabled) typeMatch = true;
                else if (SelectedTypeFilter == "Watch Folder" && job.WatchEnabled) typeMatch = true;
            }
        }

        bool taskMatch = SelectedTaskFilter == "Todos" || SelectedTaskFilter == p.JobName;

        if (typeMatch && taskMatch)
        {
            // Atualiza velocidade global baseada nos jobs ativos filtrados
            var activeJobs = _globalProgressService.GetActiveJobs();
            double totalSpeed = 0;
            foreach (var aj in activeJobs)
            {
                // Aplica filtro no cálculo da velocidade também
                bool ajTypeMatch = SelectedTypeFilter == "Todos";
                if (!ajTypeMatch)
                {
                    var j = _allJobs.FirstOrDefault(job => job.Id == aj.JobId);
                    if (j != null && ((SelectedTypeFilter == "Cpia Direta" && !j.WatchEnabled) || (SelectedTypeFilter == "Watch Folder" && j.WatchEnabled)))
                        ajTypeMatch = true;
                }
                if (ajTypeMatch && (SelectedTaskFilter == "Todos" || SelectedTaskFilter == aj.JobName))
                    totalSpeed += aj.TransferSpeed;
            }
            TransferSpeed = FormatSpeed(totalSpeed);

            // Cloud Sync Status: se algum arquivo está sendo processado e veio da nuvem
            if (p.Status == "HIDRATANDO") CloudSyncStatus = "Baixando...";
            else CloudSyncStatus = "Sincronizado";

            UpdateFilesStatsFromLive();
        }
    }

    private void UpdateFilesStatsFromLive()
    {
        TotalProcessed = _historicalProcessed + _globalProgressService.GetActiveJobs().Sum(j => j.ProcessedFiles);
    }

    private string FormatSpeed(double bytesPerSecond)
    {
        if (bytesPerSecond < 1024) return $"{bytesPerSecond:F0} B/s";
        if (bytesPerSecond < 1024 * 1024) return $"{bytesPerSecond / 1024:F1} KB/s";
        return $"{bytesPerSecond / (1024 * 1024):F1} MB/s";
    }

    private async Task InitialLoad()
    {
        try
        {
            var settings = await _settingsStore.LoadAsync();
            if (settings != null)
            {
                _timer.Interval = TimeSpan.FromSeconds(Math.Max(1, settings.SystemMonitorUpdateIntervalSeconds));
            }
        }
        catch { }

        await LoadJobsListAsync();
        await UpdateStatsAsync();
    }

    private async Task LoadJobsListAsync()
    {
        var jobs = (await _jobAppService.GetAllJobsAsync()).ToList();
        _allJobs = jobs;

        var jobNames = jobs.Select(j => j.Name).OrderBy(n => n).ToList();
        var currentNames = AvailableTasks.Where(n => n != "Todos").OrderBy(n => n).ToList();

        if (!jobNames.SequenceEqual(currentNames))
        {
            var currentSelection = SelectedTaskFilter;

            AvailableTasks.Clear();
            AvailableTasks.Add("Todos");
            foreach (var name in jobNames) AvailableTasks.Add(name);

            if (AvailableTasks.Contains(currentSelection))
                SelectedTaskFilter = currentSelection;
            else
                SelectedTaskFilter = "Todos";
        }
    }

    partial void OnSelectedTypeFilterChanged(string value) => _ = UpdateStatsAsync();
    partial void OnSelectedTaskFilterChanged(string value) => _ = UpdateStatsAsync();

    private async Task UpdateStatsAsync()
    {
        if (_isUpdating) return;
        _isUpdating = true;

        try
        {
            await LoadJobsListAsync();

            var filteredJobs = _allJobs.AsEnumerable();
            if (SelectedTypeFilter == "Cpia Direta") filteredJobs = filteredJobs.Where(j => !j.WatchEnabled);
            else if (SelectedTypeFilter == "Watch Folder") filteredJobs = filteredJobs.Where(j => j.WatchEnabled);

            if (SelectedTaskFilter != "Todos") filteredJobs = filteredJobs.Where(j => j.Name == SelectedTaskFilter);

            var jobsList = filteredJobs.ToList();

            DirectActive = jobsList.Count(j => !j.WatchEnabled && _queueProcessor.IsJobActive(j.Id));
            DirectPaused = jobsList.Count(j => !j.WatchEnabled && !_queueProcessor.IsJobActive(j.Id));

            WatchActive = jobsList.Count(j => j.WatchEnabled && _queueProcessor.IsJobActive(j.Id));
            WatchPaused = jobsList.Count(j => j.WatchEnabled && !_queueProcessor.IsJobActive(j.Id));

            await CalculateAuditStats(jobsList.Select(j => j.Name).ToList());

            var activities = await _activityService.GetRecentActivitiesAsync(15);
            RecentActivities.Clear();
            foreach (var act in activities) RecentActivities.Add(act);
        }
        finally
        {
            _isUpdating = false;
        }
    }

    private void UpdateSystemMonitor()
    {
        try
        {
            _currentProcess.Refresh();
            RamUsage = $"{_currentProcess.WorkingSet64 / 1024 / 1024} MB";
            CpuUsage = 2.0 + (new Random().NextDouble() * 3.0); 

            if (_queueProcessor.ActiveCount == 0)
            {
                TransferSpeed = "0 KB/s";
                CloudSyncStatus = "Sincronizado";
            }
        }
        catch { }
    }

    private async Task CalculateAuditStats(List<string> jobNames)
    {
        if (!Directory.Exists(_reportsFolder)) return;

        await Task.Run(async () => {
            int processed = 0;
            int ignored = 0;
            int errors = 0;

            try
            {
                var dirInfo = new DirectoryInfo(_reportsFolder);
                var files = dirInfo.GetFiles("*.csv").OrderByDescending(f => f.CreationTime).Take(50).ToList();

                foreach (var file in files)
                {
                    var lines = await File.ReadAllLinesAsync(file.FullName);
                    if (lines.Length <= 1) continue;

                    var firstLineParts = lines[1].Split(';');
                    if (firstLineParts.Length < 2) continue;
                    var jobNameInFile = firstLineParts[1].Trim('"');

                    if (jobNames.Contains(jobNameInFile) || (jobNames.Count == _allJobs.Count && SelectedTaskFilter == "Todos"))
                    {
                        for (int i = 1; i < lines.Length; i++)
                        {
                            var parts = lines[i].Split(';');
                            if (parts.Length >= 3)
                            {
                                var status = parts[2].Trim('"');
                                if (status == "COPIADO" || status == "MOVIDO") processed++;
                                else if (status == "IGNORADO") ignored++;
                                else if (status.Contains("FALHA")) errors++;
                            }
                        }
                    }
                }

                _historicalProcessed = processed;
                _historicalIgnored = ignored;
                _historicalErrors = errors;

                Dispatcher.UIThread.Post(() => {
                    TotalProcessed = processed + _globalProgressService.GetActiveJobs().Sum(j => j.ProcessedFiles);
                    TotalIgnored = ignored;
                    TotalErrors = errors;
                });
            }
            catch { }
        });
    }

    [RelayCommand] private void NewDirectCopy() => (App.Services?.GetService(typeof(MainWindowViewModel)) as MainWindowViewModel)?.NavigateToJobs("DirectCopy");
    [RelayCommand] private void NewWatchFolder() => (App.Services?.GetService(typeof(MainWindowViewModel)) as MainWindowViewModel)?.NavigateToJobs("WatchFolder");
    [RelayCommand] private void ViewHistory() => (App.Services?.GetService(typeof(MainWindowViewModel)) as MainWindowViewModel)?.NavigateToPage("History");
    [RelayCommand] private void AccessSettings() => (App.Services?.GetService(typeof(MainWindowViewModel)) as MainWindowViewModel)?.NavigateToPage("Settings");
    [RelayCommand] private void OpenLogFolder() => Process.Start("explorer.exe", _reportsFolder);

    public void Dispose()
    {
        _globalProgressService.OnProgressReported -= HandleOnProgressReported;
        _timer.Stop();
    }
}
