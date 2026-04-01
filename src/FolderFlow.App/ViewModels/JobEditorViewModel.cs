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
    public ObservableCollection<MonitoringMode> MonitoringModes { get; } = new(new[] { MonitoringMode.RealTime, MonitoringMode.Polling });
    public ObservableCollection<FolderFlow.Domain.Enums.NotificationTrigger> NotificationTriggers { get; } = new(Enum.GetValues<FolderFlow.Domain.Enums.NotificationTrigger>());
    public ObservableCollection<ArchiveFormat> ArchiveFormats { get; } = new(Enum.GetValues<ArchiveFormat>());
    public ObservableCollection<RetentionPolicy> RetentionPolicies { get; } = new(Enum.GetValues<RetentionPolicy>());

    public event Action? Saved;
    public event Action? Cancelled;

    public JobEditorViewModel(JobAppService jobAppService, IStorageService storageService, PreviewEngine previewEngine)
    {
        _jobAppService = jobAppService;
        _storageService = storageService;
        _previewEngine = previewEngine;
    }

    [ObservableProperty]
    private bool _isWatchMode;

    [ObservableProperty]
    private int _scheduleHour;

    [ObservableProperty]
    private int _scheduleMinute;

    [ObservableProperty]
    private int _scheduleSecond;

    [ObservableProperty]
    private bool _isSun, _isMon, _isTue, _isWed, _isThu, _isFri, _isSat;

    [ObservableProperty]
    private DateTimeOffset? _selectedDateOffset;

    // Fase 1 - Webhooks e Scripts
    [ObservableProperty]
    private string _webhookUrl = string.Empty;

    [ObservableProperty]
    private FolderFlow.Domain.Enums.NotificationTrigger _notifyOn = FolderFlow.Domain.Enums.NotificationTrigger.None;

    [ObservableProperty]
    private string _preScriptPath = string.Empty;

    [ObservableProperty]
    private string _postScriptPath = string.Empty;

    // Fase 2 - Segurana, Reteno e Otimizao
    [ObservableProperty]
    private ArchiveFormat _archiveFormat = ArchiveFormat.None;

    [ObservableProperty]
    private string _encryptionKey = string.Empty;

    [ObservableProperty]
    private RetentionPolicy _retentionPolicy = RetentionPolicy.None;

    [ObservableProperty]
    private int _retentionCount = 0;

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
            MonitoringMode = job.MonitoringMode,
            ScanIntervalSeconds = job.ScanIntervalSeconds,
            NameRegex = job.NameRegex,
            MinSizeKB = job.MinSizeKB,
            MaxSizeKB = job.MaxSizeKB,
            ModifiedWithinDays = job.ModifiedWithinDays,
            ScheduleType = job.ScheduleType,
            IntervalMinutes = job.IntervalMinutes,
            ScheduleTime = job.ScheduleTime,
            SpecificDate = job.SpecificDate,
            WebhookUrl = job.WebhookUrl,
            NotifyOn = job.NotifyOn,
            PreScriptPath = job.PreScriptPath,
            PostScriptPath = job.PostScriptPath,
            ArchiveFormat = job.ArchiveFormat,
            EncryptionKey = job.EncryptionKey,
            RetentionPolicy = job.RetentionPolicy,
            RetentionCount = job.RetentionCount,
            DaysOfWeek = new System.Collections.Generic.List<DayOfWeek>(job.DaysOfWeek ?? new()),
            IncludeExtensions = new System.Collections.Generic.List<string>(job.IncludeExtensions),
            ExcludePatterns = new System.Collections.Generic.List<string>(job.ExcludePatterns)
        };
        SourcePathText = Job.SourcePath;
        TargetPathText = Job.TargetPath;
        ExtensionsText = string.Join(", ", Job.IncludeExtensions);
        ExcludePatternsText = string.Join(", ", Job.ExcludePatterns);
        SelectedDateOffset = Job.SpecificDate.HasValue ? new DateTimeOffset(Job.SpecificDate.Value) : null;
        
        WebhookUrl = Job.WebhookUrl ?? string.Empty;
        NotifyOn = Job.NotifyOn;
        PreScriptPath = Job.PreScriptPath ?? string.Empty;
        PostScriptPath = Job.PostScriptPath ?? string.Empty;
        ArchiveFormat = Job.ArchiveFormat;
        EncryptionKey = Job.EncryptionKey ?? string.Empty;
        RetentionPolicy = Job.RetentionPolicy;
        RetentionCount = Job.RetentionCount;

        var days = Job.DaysOfWeek ?? new();
        IsSun = days.Contains(DayOfWeek.Sunday);
        IsMon = days.Contains(DayOfWeek.Monday);
        IsTue = days.Contains(DayOfWeek.Tuesday);
        IsWed = days.Contains(DayOfWeek.Wednesday);
        IsThu = days.Contains(DayOfWeek.Thursday);
        IsFri = days.Contains(DayOfWeek.Friday);
        IsSat = days.Contains(DayOfWeek.Saturday);

        if (TimeSpan.TryParse(Job.ScheduleTime, out var ts))
        {
            ScheduleHour = ts.Hours;
            ScheduleMinute = ts.Minutes;
            ScheduleSecond = ts.Seconds;
        }
        else
        {
            ScheduleHour = 3; ScheduleMinute = 0; ScheduleSecond = 0;
        }

        CurrentPreview = null;
    }

    private void SyncJobFields()
    {
        Job.SourcePath = SourcePathText;
        Job.TargetPath = TargetPathText;
        Job.SpecificDate = SelectedDateOffset.HasValue ? SelectedDateOffset.Value.DateTime : null;
        Job.ScheduleTime = $"{ScheduleHour:D2}:{ScheduleMinute:D2}:{ScheduleSecond:D2}";
        
        Job.WebhookUrl = string.IsNullOrWhiteSpace(WebhookUrl) ? null : WebhookUrl.Trim();
        Job.NotifyOn = NotifyOn;
        Job.PreScriptPath = string.IsNullOrWhiteSpace(PreScriptPath) ? null : PreScriptPath.Trim();
        Job.PostScriptPath = string.IsNullOrWhiteSpace(PostScriptPath) ? null : PostScriptPath.Trim();
        
        Job.ArchiveFormat = ArchiveFormat;
        Job.EncryptionKey = string.IsNullOrWhiteSpace(EncryptionKey) ? null : EncryptionKey;
        Job.RetentionPolicy = RetentionPolicy;
        Job.RetentionCount = RetentionCount;

        var days = new System.Collections.Generic.List<DayOfWeek>();
        if (IsSun) days.Add(DayOfWeek.Sunday);
        if (IsMon) days.Add(DayOfWeek.Monday);
        if (IsTue) days.Add(DayOfWeek.Tuesday);
        if (IsWed) days.Add(DayOfWeek.Wednesday);
        if (IsThu) days.Add(DayOfWeek.Thursday);
        if (IsFri) days.Add(DayOfWeek.Friday);
        if (IsSat) days.Add(DayOfWeek.Saturday);
        Job.DaysOfWeek = days;

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

            // Validao de Segurana e Compliance
            if (!FolderFlow.Application.Helpers.PathValidator.IsValidPath(Job.SourcePath, out var errorSource))
            {
                var ns = App.Services?.GetService(typeof(INotificationService)) as INotificationService;
                ns?.Show("Caminho Invlido", $"Origem: {errorSource}", true);
                return;
            }

            if (!FolderFlow.Application.Helpers.PathValidator.IsValidPath(Job.TargetPath, out var errorTarget))
            {
                var ns = App.Services?.GetService(typeof(INotificationService)) as INotificationService;
                ns?.Show("Caminho Invlido", $"Destino: {errorTarget}", true);
                return;
            }

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
