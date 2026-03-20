using System;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using FolderFlow.Application.Interfaces;
using FolderFlow.Application.Services;
using FolderFlow.Domain.Entities;
using FolderFlow.Domain.Enums;
using FolderFlow.Domain.ValueObjects;

namespace FolderFlow.App.ViewModels;

public partial class JobEditorViewModel : ViewModelBase
{
    private readonly JobAppService _jobAppService;
    private readonly IStorageService _storageService;
    private readonly PreviewEngine _previewEngine;

    [ObservableProperty]
    private Job _job = new();

    [ObservableProperty]
    private string _sourcePathText = string.Empty;

    [ObservableProperty]
    private string _targetPathText = string.Empty;

    [ObservableProperty]
    private string _extensionsText = string.Empty;

    [ObservableProperty]
    private string _excludePatternsText = string.Empty;

    [ObservableProperty]
    private PreviewSummary? _currentPreview;

    [ObservableProperty]
    private bool _isPreviewLoading;

    public ObservableCollection<JobMode> Modes { get; } = new(new[] { JobMode.Copy, JobMode.Move });
    public ObservableCollection<ConflictMode> ConflictModes { get; } = new(new[] { ConflictMode.Skip, ConflictMode.Overwrite, ConflictMode.Rename });
    public ObservableCollection<ScheduleType> ScheduleTypes { get; } = new(new[] { ScheduleType.None, ScheduleType.Interval, ScheduleType.Daily, ScheduleType.Weekly });

    public event Action? Saved;
    public event Action? Cancelled;

    public JobEditorViewModel(JobAppService jobAppService, IStorageService storageService, PreviewEngine previewEngine)
    {
        _jobAppService = jobAppService;
        _storageService = storageService;
        _previewEngine = previewEngine;
    }

    public void SetJob(Job job)
    {
        Job = new Job
        {
            Id = job.Id,
            Name = job.Name,
            SourcePath = job.SourcePath,
            TargetPath = job.TargetPath,
            Mode = job.Mode,
            Recursive = job.Recursive,
            ConflictMode = job.ConflictMode,
            SmartSync = job.SmartSync,
            WatchEnabled = job.WatchEnabled,
            VerifyHash = job.VerifyHash,
            EnableTrash = job.EnableTrash,
            SettleTimeSeconds = job.SettleTimeSeconds,
            NameRegex = job.NameRegex,
            MinSizeKB = job.MinSizeKB,
            MaxSizeKB = job.MaxSizeKB,
            ModifiedWithinDays = job.ModifiedWithinDays,
            ScheduleType = job.ScheduleType,
            IntervalMinutes = job.IntervalMinutes,
            ScheduleTime = job.ScheduleTime,
            IncludeExtensions = new System.Collections.Generic.List<string>(job.IncludeExtensions),
            ExcludePatterns = new System.Collections.Generic.List<string>(job.ExcludePatterns)
        };
        SourcePathText = Job.SourcePath;
        TargetPathText = Job.TargetPath;
        ExtensionsText = string.Join(", ", Job.IncludeExtensions);
        ExcludePatternsText = string.Join(", ", Job.ExcludePatterns);
        CurrentPreview = null;
    }

    private void SyncJobFields()
    {
        Job.SourcePath = SourcePathText;
        Job.TargetPath = TargetPathText;
        Job.IncludeExtensions = ExtensionsText.Split(',', StringSplitOptions.RemoveEmptyEntries).Select(s => s.Trim()).ToList();
        Job.ExcludePatterns = ExcludePatternsText.Split(',', StringSplitOptions.RemoveEmptyEntries).Select(s => s.Trim()).ToList();
    }

    [RelayCommand]
    private async Task GeneratePreview()
    {
        IsPreviewLoading = true;
        try
        {
            SyncJobFields();
            CurrentPreview = await _previewEngine.GeneratePreviewAsync(Job);
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Erro ao gerar preview: {ex.Message}");
        }
        finally
        {
            IsPreviewLoading = false;
        }
    }

    [RelayCommand]
    private async Task SelectSourcePath()
    {
        var path = await _storageService.SelectFolderAsync();
        if (path != null) SourcePathText = path;
    }

    [RelayCommand]
    private async Task SelectTargetPath()
    {
        var path = await _storageService.SelectFolderAsync();
        if (path != null) TargetPathText = path;
    }

    [RelayCommand]
    private async Task Save()
    {
        try
        {
            SyncJobFields();
            await _jobAppService.SaveJobAsync(Job);
            Saved?.Invoke();
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Erro ao salvar job: {ex.Message}");
        }
    }

    [RelayCommand]
    private void Cancel() => Cancelled?.Invoke();
}
