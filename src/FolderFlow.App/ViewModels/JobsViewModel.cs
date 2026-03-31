using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using FolderFlow.Application.Interfaces;
using FolderFlow.Application.Services;
using FolderFlow.Domain.Entities;
using FolderFlow.Domain.ValueObjects;

namespace FolderFlow.App.ViewModels;

public partial class JobsViewModel : ViewModelBase
{
    private readonly JobAppService _jobAppService;
    private readonly IJobQueue _jobQueue;
    private readonly QueueProcessor _queueProcessor;
    private readonly WatchAppService _watchAppService;
    private readonly IStorageService _storageService;
    private readonly ISystemActivityService _activityService;

    [ObservableProperty]
    private ObservableCollection<JobItemViewModel> _jobs = new();

    [ObservableProperty]
    private string _searchText = string.Empty;

    [ObservableProperty]
    private bool _isEditing;

    [ObservableProperty]
    private bool _isRunning;

    [ObservableProperty]
    private JobProgressInfo? _currentJobProgress;

    [ObservableProperty]
    private JobEditorViewModel _editor;

    [ObservableProperty]
    private string _selectedOperationFilter = "Todos";

    [ObservableProperty]
    private string _selectedTaskFilter = "Todos";

    public ObservableCollection<string> AvailableOperations { get; } = new(new[] { "Todos", "COPIADO", "MOVIDO", "IGNORADO", "FALHA", "CANCELADO" });
    public ObservableCollection<string> AvailableTasks { get; } = new(new[] { "Todos" });

    [ObservableProperty]
    private ObservableCollection<JobLogViewModel> _operationLogs = new();

    private List<JobLogViewModel> _allLogs = new();

    private readonly System.Collections.Concurrent.ConcurrentDictionary<Guid, CancellationTokenSource> _activeTasks = new();

    public IEnumerable<JobItemViewModel> FilteredDirectCopyJobs =>
        Jobs.Where(j => !j.Job.WatchEnabled && MatchSearch(j.Job));

    public IEnumerable<JobItemViewModel> FilteredWatchFolderJobs =>
        Jobs.Where(j => j.Job.WatchEnabled && MatchSearch(j.Job));

    public JobsViewModel(
        JobAppService jobAppService, 
        IJobQueue jobQueue, 
        QueueProcessor queueProcessor,
        WatchAppService watchAppService,
        IStorageService storageService,
        JobEditorViewModel editor,
        ISystemActivityService activityService)
    {
        _jobAppService = jobAppService;
        _jobQueue = jobQueue;
        _queueProcessor = queueProcessor;
        _watchAppService = watchAppService;
        _storageService = storageService;
        _editor = editor;
        _activityService = activityService;
        
        _editor.Saved += OnEditorSaved;
        _editor.Cancelled += OnEditorCancelled;

        LoadJobsCommand.Execute(null);
    }

    private void UpdateTaskFilters()
    {
        var current = SelectedTaskFilter;
        AvailableTasks.Clear();
        AvailableTasks.Add("Todos");
        foreach (var job in Jobs) AvailableTasks.Add(job.Job.Name);
        if (AvailableTasks.Contains(current)) SelectedTaskFilter = current;
    }

    partial void OnSearchTextChanged(string value)
    {
        OnPropertyChanged(nameof(FilteredDirectCopyJobs));
        OnPropertyChanged(nameof(FilteredWatchFolderJobs));
    }

    partial void OnSelectedOperationFilterChanged(string value) => ApplyLogFilters();
    partial void OnSelectedTaskFilterChanged(string value) => ApplyLogFilters();

    private void ApplyLogFilters()
    {
        var filtered = _allLogs.AsEnumerable();
        if (SelectedOperationFilter != "Todos")
            filtered = filtered.Where(l => l.Progress.Status.Contains(SelectedOperationFilter, StringComparison.OrdinalIgnoreCase));
        if (SelectedTaskFilter != "Todos")
            filtered = filtered.Where(l => l.Progress.JobName == SelectedTaskFilter);

        OperationLogs.Clear();
        foreach (var log in filtered) OperationLogs.Add(log);
    }

    private bool MatchSearch(Job job)
    {
        if (string.IsNullOrWhiteSpace(SearchText)) return true;
        return job.Name.Contains(SearchText, StringComparison.OrdinalIgnoreCase) ||
               job.SourcePath.Contains(SearchText, StringComparison.OrdinalIgnoreCase) ||
               job.TargetPath.Contains(SearchText, StringComparison.OrdinalIgnoreCase);
    }

    [RelayCommand]
    private async Task LoadJobs()
    {
        var jobsList = await _jobAppService.GetAllJobsAsync();
        Jobs.Clear();
        foreach (var job in jobsList)
        {
            Jobs.Add(new JobItemViewModel(job));
        }
        UpdateTaskFilters();
        OnSearchTextChanged(SearchText);
    }

    [RelayCommand]
    private void CreateDirectCopyJob()
    {
        var job = new Job { WatchEnabled = false };
        Editor.SetJob(job);
        Editor.IsWatchMode = false;
        IsEditing = true;
    }

    [RelayCommand]
    private void CreateWatchFolderJob()
    {
        var job = new Job { WatchEnabled = true };
        Editor.SetJob(job);
        Editor.IsWatchMode = true;
        IsEditing = true;
    }

    [RelayCommand]
    private void EditJob(Job job)
    {
        if (job == null) return;
        Editor.SetJob(job);
        Editor.IsWatchMode = job.WatchEnabled;
        IsEditing = true;
    }

    [RelayCommand]
    private async Task DeleteJob(Job job)
    {
        if (job == null) return;
        await _jobAppService.DeleteJobAsync(job.Id);
        job.WatchEnabled = false;
        _watchAppService.UpdateJobWatching(job);
        await _activityService.LogActivityAsync($"Tarefa '{job.Name}' excluda.", "WARNING");
        await LoadJobs();
    }

    [RelayCommand]
    private async Task RunJob(Job job) => await RunJobInternal(job, false);

    [RelayCommand]
    private async Task RetryJobFailures(Job job) => await RunJobInternal(job, true);

    [RelayCommand]
    private async Task RunAll()
    {
        var selected = Jobs.Where(j => j.IsSelected).ToList();
        var targetJobs = selected.Any() ? selected.Select(j => j.Job) : Jobs.Select(j => j.Job);
        
        foreach (var job in targetJobs)
        {
            _ = RunJobInternal(job, false);
        }
    }

    [RelayCommand]
    private void StopAll()
    {
        var selectedIds = Jobs.Where(j => j.IsSelected).Select(j => j.Job.Id).ToHashSet();
        
        foreach (var active in _activeTasks)
        {
            if (!selectedIds.Any() || selectedIds.Contains(active.Key))
            {
                active.Value.Cancel();
            }
        }

        foreach (var job in Jobs)
        {
             if (!selectedIds.Any() || selectedIds.Contains(job.Job.Id))
             {
                 _queueProcessor.StopJob(job.Job.Id);
             }
        }
    }

    [RelayCommand]
    private void ClearLogs()
    {
        _allLogs.Clear();
        OperationLogs.Clear();
    }

    [RelayCommand]
    private async Task StopJob(Guid jobId)
    {
        if (_activeTasks.TryGetValue(jobId, out var cts))
        {
            cts.Cancel();
        }
        _queueProcessor.StopJob(jobId);

        var jobItem = Jobs.FirstOrDefault(j => j.Job.Id == jobId);
        if (jobItem != null)
        {
            jobItem.Job.WatchEnabled = false;
            await _jobAppService.SaveJobAsync(jobItem.Job);
            _watchAppService.UpdateJobWatching(jobItem.Job);
            await _activityService.LogActivityAsync($"Tarefa '{jobItem.Job.Name}' parada/pausada.", "INFO");
        }

        await LoadJobs();
    }

    private async Task RunJobInternal(Job job, bool isRetry)
    {
        if (job == null) return;
        if (_activeTasks.ContainsKey(job.Id)) return;

        var cts = new CancellationTokenSource();
        _activeTasks.TryAdd(job.Id, cts);

        try
        {
            IsRunning = true;
            var engine = App.Services?.GetService(typeof(ExecutionEngine)) as ExecutionEngine;
            if (engine != null)
            {
                var progress = new Progress<JobProgressInfo>(p => 
                {
                    CurrentJobProgress = p;
                    if (!string.IsNullOrEmpty(p.Status))
                    {
                        Avalonia.Threading.Dispatcher.UIThread.Post(() => 
                        {
                            var existing = _allLogs.FirstOrDefault(l => l.Progress.JobName == p.JobName && l.Progress.CurrentFile == p.CurrentFile && (DateTime.Now - l.Progress.Timestamp).TotalSeconds < 10);
                            if (existing != null)
                            {
                                existing.Update(p);
                            }
                            else
                            {
                                var newLog = new JobLogViewModel(new JobProgressInfo 
                                { 
                                    JobName = p.JobName, 
                                    CurrentFile = p.CurrentFile, 
                                    Status = p.Status, 
                                    ProcessedFiles = p.ProcessedFiles,
                                    TotalFiles = p.TotalFiles,
                                    CurrentFilePercentage = p.CurrentFilePercentage,
                                    TransferSpeed = p.TransferSpeed,
                                    Timestamp = DateTime.Now
                                });
                                _allLogs.Insert(0, newLog);
                                if (_allLogs.Count > 500) _allLogs.RemoveAt(500);
                                ApplyLogFilters();
                            }
                        });
                    }
                });

                await Task.Run(async () => 
                {
                    await engine.RunJobAsync(job, cts.Token, isRetry, progress);
                }, cts.Token);
            }
        }
        catch (OperationCanceledException)
        {
            Avalonia.Threading.Dispatcher.UIThread.Post(() => 
            {
                var log = new JobLogViewModel(new JobProgressInfo { JobName = job.Name, Status = "CANCELADO", Timestamp = DateTime.Now });
                _allLogs.Insert(0, log);
                ApplyLogFilters();
            });
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Erro ao rodar (Retry={isRetry}): {ex.Message}");
        }
        finally
        {
            _activeTasks.TryRemove(job.Id, out _);
            IsRunning = _activeTasks.Any();
            CurrentJobProgress = null;
            await LoadJobs();
        }
    }

    [RelayCommand]
    private async Task RunSelectedJobs()
    {
        var selected = Jobs.Where(j => j.IsSelected).ToList();
        foreach (var item in selected)
        {
            await _jobQueue.EnqueueAsync(item.Job);
            item.IsSelected = false; 
        }
    }

    [RelayCommand]
    private async Task PauseSelectedJobs()
    {
        var selectedWatchers = Jobs.Where(j => j.IsSelected && j.Job.WatchEnabled).ToList();
        foreach (var item in selectedWatchers)
        {
            item.Job.WatchEnabled = false;
            await _jobAppService.SaveJobAsync(item.Job);
            _watchAppService.UpdateJobWatching(item.Job);
            item.IsSelected = false;
            await _activityService.LogActivityAsync($"Tarefa '{item.Job.Name}' pausada.", "INFO");
        }
        await LoadJobs();
    }

    [RelayCommand]
    private async Task ExportSelectedJobs()
    {
        var selected = Jobs.Where(j => j.IsSelected).Select(j => j.Job).ToList();
        if (!selected.Any()) return;

        var path = await _storageService.SaveFileAsync("backup_jobs.json", "FolderFlow Jobs", "json");
        if (path != null)
        {
            await _jobAppService.ExportJobsAsync(selected, path);
        }
    }

    [RelayCommand]
    private async Task ImportJobs()
    {
        var path = await _storageService.OpenFileAsync("FolderFlow Jobs", "json");
        if (path != null)
        {
            await _jobAppService.ImportJobsAsync(path);
            await _activityService.LogActivityAsync("Novas tarefas importadas para o sistema.");
            await LoadJobs();
        }
    }

    private void OnEditorSaved()
    {
        IsEditing = false;
        _watchAppService.UpdateJobWatching(Editor.Job);
        _ = _activityService.LogActivityAsync($"Tarefa '{Editor.Job.Name}' salva/atualizada.");
        LoadJobsCommand.Execute(null);
    }

    private void OnEditorCancelled()
    {
        IsEditing = false;
    }
}
