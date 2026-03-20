using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
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
    private readonly WatchAppService _watchAppService;
    private readonly IStorageService _storageService;

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

    public IEnumerable<JobItemViewModel> FilteredDirectCopyJobs =>
        Jobs.Where(j => !j.Job.WatchEnabled && MatchSearch(j.Job));

    public IEnumerable<JobItemViewModel> FilteredWatchFolderJobs =>
        Jobs.Where(j => j.Job.WatchEnabled && MatchSearch(j.Job));

    public JobsViewModel(
        JobAppService jobAppService, 
        IJobQueue jobQueue, 
        WatchAppService watchAppService,
        IStorageService storageService,
        JobEditorViewModel editor)
    {
        _jobAppService = jobAppService;
        _jobQueue = jobQueue;
        _watchAppService = watchAppService;
        _storageService = storageService;
        _editor = editor;
        
        _editor.Saved += OnEditorSaved;
        _editor.Cancelled += OnEditorCancelled;

        LoadJobsCommand.Execute(null);
    }

    partial void OnSearchTextChanged(string value)
    {
        OnPropertyChanged(nameof(FilteredDirectCopyJobs));
        OnPropertyChanged(nameof(FilteredWatchFolderJobs));
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
        OnSearchTextChanged(SearchText);
    }

    [RelayCommand]
    private void CreateNewJob()
    {
        Editor.SetJob(new Job());
        IsEditing = true;
    }

    [RelayCommand]
    private void EditJob(Job job)
    {
        if (job == null) return;
        Editor.SetJob(job);
        IsEditing = true;
    }

    [RelayCommand]
    private async Task DeleteJob(Job job)
    {
        if (job == null) return;
        await _jobAppService.DeleteJobAsync(job.Id);
        job.WatchEnabled = false;
        _watchAppService.UpdateJobWatching(job);
        await LoadJobs();
    }

    [RelayCommand]
    private async Task RunJob(Job job)
    {
        if (job == null) return;
        try
        {
            IsRunning = true;
            var engine = App.Services?.GetService(typeof(ExecutionEngine)) as ExecutionEngine;
            if (engine != null)
            {
                var progress = new Progress<JobProgressInfo>(p => CurrentJobProgress = p);
                await engine.RunJobAsync(job, default, false, progress);
            }
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Erro ao rodar: {ex.Message}");
        }
        finally
        {
            IsRunning = false;
            CurrentJobProgress = null;
        }
    }

    [RelayCommand]
    private async Task RetryJob(Job job)
    {
        if (job == null) return;
        try
        {
            IsRunning = true;
            var engine = App.Services?.GetService(typeof(ExecutionEngine)) as ExecutionEngine;
            if (engine != null)
            {
                var progress = new Progress<JobProgressInfo>(p => CurrentJobProgress = p);
                await engine.RunJobAsync(job, default, isRetry: true, progress);
            }
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Erro no retry: {ex.Message}");
        }
        finally
        {
            IsRunning = false;
            CurrentJobProgress = null;
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
            await LoadJobs();
        }
    }

    private void OnEditorSaved()
    {
        IsEditing = false;
        _watchAppService.UpdateJobWatching(Editor.Job);
        LoadJobsCommand.Execute(null);
    }

    private void OnEditorCancelled()
    {
        IsEditing = false;
    }
}
