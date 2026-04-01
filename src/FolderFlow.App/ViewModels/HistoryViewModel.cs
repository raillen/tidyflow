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

namespace FolderFlow.App.ViewModels;

public partial class HistoryViewModel : ViewModelBase
{
    private readonly IAuditService _auditService;
    private readonly JobAppService _jobAppService;

    [ObservableProperty] private ObservableCollection<AuditEntryViewModel> _logs = new();
    [ObservableProperty] private AuditEntryViewModel? _selectedLog;
    [ObservableProperty] private string _searchText = string.Empty;
    [ObservableProperty] private string _selectedJobFilter = "Todos";
    [ObservableProperty] private string _selectedStatusFilter = "Todos";

    public ObservableCollection<string> AvailableJobs { get; } = new();
    public ObservableCollection<string> AvailableStatuses { get; } = new(new[] { "Todos", "COPIADO", "MOVIDO", "IGNORADO", "FALHA" });

    public HistoryViewModel(IAuditService auditService, JobAppService jobAppService)
    {
        _auditService = auditService;
        _jobAppService = jobAppService;
        
        AvailableJobs.Add("Todos");
        _ = InitialLoad();
    }

    private async Task InitialLoad()
    {
        var jobs = await _jobAppService.GetAllJobsAsync();
        foreach (var job in jobs.OrderBy(j => j.Name)) AvailableJobs.Add(job.Name);
        await LoadLogs();
    }

    [RelayCommand]
    private async Task LoadLogs()
    {
        try
        {
            var sqliteAudit = _auditService as SqliteAuditService;
            if (sqliteAudit == null) return;

            var entries = await sqliteAudit.GetLogsAsync(SelectedJobFilter, SelectedStatusFilter, SearchText);
            
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
