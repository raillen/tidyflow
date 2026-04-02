using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using FolderFlow.Application.Interfaces;
using FolderFlow.Application.Services;
using FolderFlow.Infrastructure.Logging;
using System.Diagnostics;
using System.Text;
using System.Threading;

namespace FolderFlow.App.ViewModels;

public partial class HistoryViewModel : ViewModelBase
{
    private readonly IAuditService _auditService;
    private readonly JobAppService _jobAppService;
    private readonly ILocalizationService _localizationService;

    [ObservableProperty] private ObservableCollection<AuditEntryViewModel> _logs = new();
    [ObservableProperty] private AuditEntryViewModel? _selectedLog;
    [ObservableProperty] private string _searchText = string.Empty;
    [ObservableProperty] private string _selectedJobFilter = "";
    [ObservableProperty] private string _selectedStatusFilter = "";

    public ObservableCollection<string> AvailableJobs { get; } = new();
    public ObservableCollection<string> AvailableStatuses { get; } = new();

    public HistoryViewModel(IAuditService auditService, JobAppService jobAppService, ILocalizationService localizationService)
    {
        _auditService = auditService;
        _jobAppService = jobAppService;
        _localizationService = localizationService;
        
        _selectedJobFilter = _localizationService["All"];
        _selectedStatusFilter = _localizationService["All"];
        AvailableJobs.Add(_localizationService["All"]);
        AvailableStatuses.Add(_localizationService["All"]);
        AvailableStatuses.Add(_localizationService["Copied"]);
        AvailableStatuses.Add(_localizationService["Moved"]);
        AvailableStatuses.Add(_localizationService["Ignored"]);
        AvailableStatuses.Add(_localizationService["FailedStatus"]);

        if (_localizationService is System.ComponentModel.INotifyPropertyChanged npc)
        {
            npc.PropertyChanged += (s, e) => {
                if (e.PropertyName == "Item" || e.PropertyName == "Item[]" || string.IsNullOrEmpty(e.PropertyName))
                {
                    UpdateLabels();
                }
            };
        }

        _ = InitialLoad();
    }

    private void UpdateLabels()
    {
        var currentJob = SelectedJobFilter;
        var currentStatus = SelectedStatusFilter;

        AvailableStatuses.Clear();
        AvailableStatuses.Add(_localizationService["All"]);
        AvailableStatuses.Add(_localizationService["Copied"]);
        AvailableStatuses.Add(_localizationService["Moved"]);
        AvailableStatuses.Add(_localizationService["Ignored"]);
        AvailableStatuses.Add(_localizationService["FailedStatus"]);

        var oldAll = AvailableJobs.FirstOrDefault(); // "Todos" or localized
        AvailableJobs.RemoveAt(0);
        AvailableJobs.Insert(0, _localizationService["All"]);

        SelectedJobFilter = (currentJob == oldAll) ? _localizationService["All"] : currentJob;
        SelectedStatusFilter = (currentStatus == oldAll) ? _localizationService["All"] : currentStatus;
        
        _ = LoadLogs();
    }

    private async Task InitialLoad()
    {
        var jobs = await _jobAppService.GetAllJobsAsync();
        foreach (var job in jobs.OrderBy(j => j.Name)) AvailableJobs.Add(job.Name);
        await LoadLogs();
    }

    private readonly SemaphoreSlim _loadLock = new(1, 1);

    [RelayCommand]
    private async Task LoadLogs()
    {
        if (!await _loadLock.WaitAsync(0)) return;
        try
        {
            var sqliteAudit = _auditService as SqliteAuditService;
            if (sqliteAudit == null) return;

            string jobFilter = SelectedJobFilter == _localizationService["All"] ? "Todos" : SelectedJobFilter;
            string statusFilter = SelectedStatusFilter == _localizationService["All"] ? "Todos" : SelectedStatusFilter;
            
            // Map localized status back to DB status if needed, but here we assume the DB uses the same strings or we handle it.
            // Actually, "COPIADO" etc are likely what's in the DB.
            if (SelectedStatusFilter == _localizationService["Copied"]) statusFilter = "COPIADO";
            else if (SelectedStatusFilter == _localizationService["Moved"]) statusFilter = "MOVIDO";
            else if (SelectedStatusFilter == _localizationService["Ignored"]) statusFilter = "IGNORADO";
            else if (SelectedStatusFilter == _localizationService["FailedStatus"]) statusFilter = "FALHA";

            var entries = await sqliteAudit.GetLogsAsync(jobFilter, statusFilter, SearchText);
            
            Logs.Clear();
            foreach (var entry in entries)
            {
                Logs.Add(new AuditEntryViewModel(entry));
            }
        }
        catch (Exception ex)
        {
            Debug.WriteLine($"Erro ao carregar logs SQLite: {ex.Message}");
        }
        finally
        {
            _loadLock.Release();
        }
    }

    partial void OnSearchTextChanged(string value) => _ = LoadLogs();
    partial void OnSelectedJobFilterChanged(string value) => _ = LoadLogs();
    partial void OnSelectedStatusFilterChanged(string value) => _ = LoadLogs();

    [RelayCommand]
    private async Task ClearLogs()
    {
        if (_auditService is SqliteAuditService sqliteAudit)
        {
            await sqliteAudit.ClearAllLogsAsync();
            Logs.Clear();
            SelectedLog = null;
        }
    }

    [RelayCommand]
    private void OpenInExplorer(string? path)
    {
        if (string.IsNullOrEmpty(path)) return;
        try 
        {
            var dir = Path.GetDirectoryName(path);
            if (!string.IsNullOrEmpty(dir) && Directory.Exists(dir)) 
                Process.Start("explorer.exe", dir);
        } catch { }
    }

    [RelayCommand]
    private async Task ExportToCsv()
    {
        var sb = new StringBuilder();
        sb.AppendLine("Timestamp;Job;Status;Origem;Destino;Tamanho;Duracao;Detalhes");
        foreach (var log in Logs)
        {
            var e = log.Entry;
            sb.AppendLine($"{e.Timestamp:yyyy-MM-dd HH:mm:ss};{e.JobName};{e.Status};{e.SourcePath};{e.TargetPath};{e.FileSize};{e.DurationMs};{e.Details}");
        }

        var path = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.Desktop), $"Export_FolderFlow_{DateTime.Now:yyyyMMdd_HHmm}.csv");
        await File.WriteAllTextAsync(path, sb.ToString(), Encoding.UTF8);
    }
}
