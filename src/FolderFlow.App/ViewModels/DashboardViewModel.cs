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

using FolderFlow.Infrastructure.Logging;

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

    // Estatsticas histricas do SQLite para soma em tempo real
    private int _sqliteProcessed;
    private int _sqliteIgnored;
    private int _sqliteErrors;

    private readonly IAuditService _auditService;
    private readonly ISchedulerService _schedulerService;
    [ObservableProperty] private ObservableCollection<UpcomingJobInfo> _upcomingJobs = new();
    [ObservableProperty] private ObservableCollection<double> _performancePoints = new(); // ltimos 60 segundos

    public DashboardViewModel(
        JobAppService jobAppService, 
        IAuditService auditService,
        ISystemActivityService activityService,
        QueueProcessor queueProcessor,
        GlobalProgressService globalProgressService,
        ISettingsStore settingsStore,
        ISchedulerService schedulerService)
    {
        _jobAppService = jobAppService;
        _auditService = auditService;
        _activityService = activityService;
        _queueProcessor = queueProcessor;
        _globalProgressService = globalProgressService;
        _settingsStore = settingsStore;
        _schedulerService = schedulerService;
        
        for (int i = 0; i < 60; i++) PerformancePoints.Add(0);
        
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
        TotalProcessed = _sqliteProcessed + _globalProgressService.GetActiveJobs().Sum(j => j.ProcessedFiles);
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

            // Radar de Agendamentos
            var upcoming = await _schedulerService.GetUpcomingJobsAsync(5);
            UpcomingJobs.Clear();
            foreach (var up in upcoming) UpcomingJobs.Add(up);

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
            // Monitoramento Real via P/Invoke e PerformanceCounter
            var ram = FolderFlow.Infrastructure.Helpers.SystemMonitor.GetRamStatus();
            CpuUsage = FolderFlow.Infrastructure.Helpers.SystemMonitor.GetCpuUsage();
            RamUsage = $"{ram.used / 1024 / 1024 / 1024} GB / {ram.total / 1024 / 1024 / 1024} GB";

            // Grfico de Performance
            var activeJobs = _globalProgressService.GetActiveJobs();
            double currentSpeed = activeJobs.Sum(j => j.TransferSpeed);
            
            Dispatcher.UIThread.Post(() => {
                PerformancePoints.Add(currentSpeed);
                if (PerformancePoints.Count > 60) PerformancePoints.RemoveAt(0);
            });

            if (_queueProcessor.ActiveCount == 0)
            {
                TransferSpeed = "0 KB/s";
                CloudSyncStatus = "Sincronizado";
            }
        }
        catch { }
    }

    // Premium Metrics
    [ObservableProperty] private string _totalDataVolume = "0 B";
    [ObservableProperty] private string _timeSaved = "0h 0m";
    [ObservableProperty] private double _healthScore = 100.0;
    [ObservableProperty] private string _healthStatus = "Excelente";

    private async Task CalculateAuditStats(List<string> jobNames)
    {
        if (_activityService is null) return; // Safety check

        try
        {
            var sqliteAudit = _auditService as SqliteAuditService;
            if (sqliteAudit == null) return;

            string? jobFilter = SelectedTaskFilter == "Todos" ? null : SelectedTaskFilter;
            
            var stats = await sqliteAudit.GetStatsAsync(jobFilter);
            var bytes = await sqliteAudit.GetTotalBytesProcessedAsync(jobFilter);

            _sqliteProcessed = stats.success;
            _sqliteIgnored = stats.ignored;
            _sqliteErrors = stats.errors;

            Dispatcher.UIThread.Post(() => {
                TotalProcessed = stats.success + _globalProgressService.GetActiveJobs().Sum(j => j.ProcessedFiles);
                TotalIgnored = stats.ignored;
                TotalErrors = stats.errors;
                TotalDataVolume = FormatSize(bytes);
                
                // Clculo de Economia de Tempo (Ex: 3 segundos por arquivo processado)
                var totalSecondsSaved = stats.success * 3; 
                var hours = totalSecondsSaved / 3600;
                var minutes = (totalSecondsSaved % 3600) / 60;
                TimeSaved = $"{hours}h {minutes}m";

                // Clculo de Health Score
                int total = stats.success + stats.errors;
                if (total > 0)
                {
                    HealthScore = (double)stats.success / total * 100.0;
                    if (HealthScore > 95) HealthStatus = "Excelente";
                    else if (HealthScore > 80) HealthStatus = "Bom";
                    else if (HealthScore > 60) HealthStatus = "Ateno";
                    else HealthStatus = "Crtico";
                }
                else
                {
                    HealthScore = 100;
                    HealthStatus = "Sem dados";
                }
            });
        }
        catch (Exception ex)
        {
            Debug.WriteLine($"Erro ao calcular stats: {ex.Message}");
        }
    }

    private string FormatSize(long bytes)
    {
        string[] units = { "B", "KB", "MB", "GB", "TB" };
        double size = bytes;
        int unitIndex = 0;
        while (size >= 1024 && unitIndex < units.Length - 1)
        {
            size /= 1024;
            unitIndex++;
        }
        return $"{size:F1} {units[unitIndex]}";
    }

    [RelayCommand] private void NewDirectCopy() => (App.Services?.GetService(typeof(MainWindowViewModel)) as MainWindowViewModel)?.NavigateToJobs("DirectCopy");
    [RelayCommand] private void NewWatchFolder() => (App.Services?.GetService(typeof(MainWindowViewModel)) as MainWindowViewModel)?.NavigateToJobs("WatchFolder");
    
    [RelayCommand] 
    private void ToggleAllJobs()
    {
        _queueProcessor.TogglePause();
        _activityService.LogActivityAsync(_queueProcessor.IsPaused ? "Processamento global pausado." : "Processamento global retomado.");
        _ = UpdateStatsAsync();
    }
    [RelayCommand] private void ViewHistory() => (App.Services?.GetService(typeof(MainWindowViewModel)) as MainWindowViewModel)?.NavigateToPage("History");
    [RelayCommand] private void AccessSettings() => (App.Services?.GetService(typeof(MainWindowViewModel)) as MainWindowViewModel)?.NavigateToPage("Settings");
    [RelayCommand] private void OpenLogFolder() => Process.Start("explorer.exe", _reportsFolder);

    public void Dispose()
    {
        _globalProgressService.OnProgressReported -= HandleOnProgressReported;
        _timer.Stop();
    }
}
