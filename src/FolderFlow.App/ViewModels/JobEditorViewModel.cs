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
    private readonly ISettingsStore _settingsStore;
    private readonly ILocalizationService _localizationService;

    [ObservableProperty] private Job _job = new();
    [ObservableProperty] private string _sourcePathText = string.Empty;
    [ObservableProperty] private string _targetPathText = string.Empty;
    [ObservableProperty] private string _extensionsText = string.Empty;
    [ObservableProperty] private string _excludePatternsText = string.Empty;
    [ObservableProperty] private PreviewSummary? _currentPreview;
    [ObservableProperty] private bool _isPreviewLoading;
    [ObservableProperty] private bool _isWatchMode;
    [ObservableProperty] private string _watchModeLabel = string.Empty;
    [ObservableProperty] private bool _hasPreview;

    // Organization
    [ObservableProperty] private ObservableCollection<string> _blueprintFolders = new();
    [ObservableProperty] private string _newFolderName = string.Empty;

    // Time Components
    [ObservableProperty] private int _scheduleHour = 3;
    [ObservableProperty] private int _scheduleMinute = 0;
    [ObservableProperty] private int _scheduleSecond = 0;
    [ObservableProperty] private DateTimeOffset? _selectedDateOffset;
    [ObservableProperty] private bool _isSun, _isMon, _isTue, _isWed, _isThu, _isFri, _isSat;

    // Enums for UI
    public ObservableCollection<JobMode> Modes { get; } = new(new[] { JobMode.Copy, JobMode.Move });
    public ObservableCollection<ConflictMode> ConflictModes { get; } = new(new[] { ConflictMode.Skip, ConflictMode.Overwrite, ConflictMode.Rename });
    public ObservableCollection<ScheduleType> ScheduleTypes { get; } = new(new[] { ScheduleType.None, ScheduleType.Interval, ScheduleType.Daily, ScheduleType.Weekly });
    public ObservableCollection<MonitoringMode> MonitoringModes { get; } = new(new[] { MonitoringMode.RealTime, MonitoringMode.Polling });
    public ObservableCollection<NotificationTrigger> NotificationTriggers { get; } = new(Enum.GetValues<NotificationTrigger>());
    public ObservableCollection<ArchiveFormat> ArchiveFormats { get; } = new(Enum.GetValues<ArchiveFormat>());
    public ObservableCollection<RetentionPolicy> RetentionPolicies { get; } = new(Enum.GetValues<RetentionPolicy>());

    public event Action? Saved;
    public event Action? Cancelled;

    public JobEditorViewModel(
        JobAppService jobAppService, 
        IStorageService storageService, 
        PreviewEngine previewEngine,
        ISettingsStore settingsStore,
        ILocalizationService localizationService)
    {
        _jobAppService = jobAppService;
        _storageService = storageService;
        _previewEngine = previewEngine;
        _settingsStore = settingsStore;
        _localizationService = localizationService;

        if (_localizationService is System.ComponentModel.INotifyPropertyChanged npc)
        {
            npc.PropertyChanged += (s, e) => {
                if (e.PropertyName == "Item" || e.PropertyName == "Item[]" || string.IsNullOrEmpty(e.PropertyName))
                {
                    UpdateWatchModeLabel();
                }
            };
        }
    }

    partial void OnIsWatchModeChanged(bool value) => UpdateWatchModeLabel();

    private void UpdateWatchModeLabel()
    {
        WatchModeLabel = IsWatchMode ? _localizationService["WatchFolder"] : _localizationService["DirectCopy"];
    }

    public async Task SetJob(Job job)
    {
        Job = job;
        IsWatchMode = job.WatchEnabled;
        UpdateWatchModeLabel();
        SourcePathText = job.SourcePath;
        TargetPathText = job.TargetPath;
        ExtensionsText = string.Join(", ", job.IncludeExtensions);
        ExcludePatternsText = string.Join(", ", job.ExcludePatterns);
        BlueprintFolders = new ObservableCollection<string>(job.BlueprintFolders);
        
        if (TimeSpan.TryParse(job.ScheduleTime, out var ts))
        {
            ScheduleHour = ts.Hours;
            ScheduleMinute = ts.Minutes;
            ScheduleSecond = ts.Seconds;
        }

        // Se for um novo Job e tiver chave mestra nas configuraes, sugere ela
        var settings = await _settingsStore.LoadAsync();
        if (string.IsNullOrEmpty(Job.EncryptionKey) && !string.IsNullOrEmpty(settings.MasterEncryptionKey))
        {
            Job.EncryptionKey = settings.MasterEncryptionKey;
        }

        CurrentPreview = null;
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

    private void SyncFields()
    {
        Job.SourcePath = SourcePathText;
        Job.TargetPath = TargetPathText;
        Job.WatchEnabled = IsWatchMode;
        Job.ScheduleTime = $"{ScheduleHour:D2}:{ScheduleMinute:D2}:{ScheduleSecond:D2}";
        Job.SpecificDate = SelectedDateOffset?.DateTime;
        
        Job.IncludeExtensions = ExtensionsText.Split(',', StringSplitOptions.RemoveEmptyEntries).Select(s => s.Trim()).ToList();
        Job.ExcludePatterns = ExcludePatternsText.Split(',', StringSplitOptions.RemoveEmptyEntries).Select(s => s.Trim()).ToList();

        var days = new System.Collections.Generic.List<DayOfWeek>();
        if (IsSun) days.Add(DayOfWeek.Sunday);
        if (IsMon) days.Add(DayOfWeek.Monday);
        if (IsTue) days.Add(DayOfWeek.Tuesday);
        if (IsWed) days.Add(DayOfWeek.Wednesday);
        if (IsThu) days.Add(DayOfWeek.Thursday);
        if (IsFri) days.Add(DayOfWeek.Friday);
        if (IsSat) days.Add(DayOfWeek.Saturday);
        Job.DaysOfWeek = days;
        Job.BlueprintFolders = BlueprintFolders.ToList();
    }

    [RelayCommand]
    private void AddBlueprintFolder()
    {
        if (!string.IsNullOrWhiteSpace(NewFolderName))
        {
            if (!BlueprintFolders.Contains(NewFolderName))
            {
                BlueprintFolders.Add(NewFolderName);
            }
            NewFolderName = string.Empty;
        }
    }

    [RelayCommand]
    private void RemoveBlueprintFolder(string folderName)
    {
        BlueprintFolders.Remove(folderName);
    }

    [RelayCommand]
    private async Task Save()
    {
        SyncFields();
        await _jobAppService.SaveJobAsync(Job);
        Saved?.Invoke();
    }

    [RelayCommand]
    private void Cancel() => Cancelled?.Invoke();

    [RelayCommand]
    private async Task GeneratePreview()
    {
        IsPreviewLoading = true;
        SyncFields();
        CurrentPreview = await _previewEngine.GeneratePreviewAsync(Job);
        HasPreview = CurrentPreview != null;
        IsPreviewLoading = false;
    }

    [RelayCommand]
    private void ClearPreview()
    {
        CurrentPreview = null;
        HasPreview = false;
    }
}
