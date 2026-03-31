using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using FolderFlow.Domain.Entities;

namespace FolderFlow.App.ViewModels;

public partial class HistoryViewModel : ViewModelBase
{
    private readonly string _reportsFolder;

    private List<AuditEntryViewModel> _allEntries = new();

    [ObservableProperty]
    private ObservableCollection<AuditEntryViewModel> _logs = new();

    [ObservableProperty]
    private string _searchText = string.Empty;

    [ObservableProperty]
    private string _selectedJobFilter = "Todos";

    [ObservableProperty]
    private string _selectedStatusFilter = "Todos";

    public ObservableCollection<string> AvailableJobs { get; } = new();
    public ObservableCollection<string> AvailableStatuses { get; } = new();

    public HistoryViewModel()
    {
        var dataFolder = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Data");
        _reportsFolder = Path.Combine(dataFolder, "Reports");
        
        AvailableStatuses.Add("Todos");
        AvailableStatuses.Add("COPIADO");
        AvailableStatuses.Add("MOVIDO");
        AvailableStatuses.Add("IGNORADO");
        AvailableStatuses.Add("FALHA");

        AvailableJobs.Add("Todos");

        LoadLogsCommand.Execute(null);
    }

    [RelayCommand]
    private async Task LoadLogs()
    {
        if (!Directory.Exists(_reportsFolder)) return;

        await Task.Run(async () =>
        {
            var allEntries = new List<AuditEntryViewModel>();
            var tempJobs = new HashSet<string>();

            try
            {
                var dirInfo = new DirectoryInfo(_reportsFolder);
                var files = dirInfo.GetFiles("*.csv").OrderByDescending(f => f.CreationTime);

                foreach (var file in files)
                {
                    var lines = await File.ReadAllLinesAsync(file.FullName);
                    // Ignora o cabeçalho (linha 0)
                    for (int i = 1; i < lines.Length; i++)
                    {
                        var line = lines[i];
                        var parts = line.Split(';');
                        if (parts.Length >= 6)
                        {
                            var entry = new AuditEntry
                            {
                                Timestamp = DateTime.TryParse(parts[0], out var dt) ? dt : DateTime.Now,
                                JobName = parts[1].Trim('"'),
                                Status = parts[2].Trim('"'),
                                SourcePath = parts[3].Trim('"'),
                                TargetPath = parts[4].Trim('"'),
                                Details = parts[5].Trim('"')
                            };

                            allEntries.Add(new AuditEntryViewModel(entry));
                            tempJobs.Add(entry.JobName);
                        }
                    }
                }

                allEntries = allEntries.OrderByDescending(e => e.Entry.Timestamp).ToList();

                Avalonia.Threading.Dispatcher.UIThread.Post(() =>
                {
                    _allEntries = allEntries;
                    
                    var currentJob = SelectedJobFilter;
                    AvailableJobs.Clear();
                    AvailableJobs.Add("Todos");
                    foreach (var job in tempJobs.OrderBy(j => j)) AvailableJobs.Add(job);

                    if (AvailableJobs.Contains(currentJob)) SelectedJobFilter = currentJob;
                    else SelectedJobFilter = "Todos";

                    ApplyFilters();
                });
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Erro ao carregar logs CSV: {ex.Message}");
            }
        });
    }

    partial void OnSearchTextChanged(string value) => ApplyFilters();
    partial void OnSelectedJobFilterChanged(string value) => ApplyFilters();
    partial void OnSelectedStatusFilterChanged(string value) => ApplyFilters();

    private void ApplyFilters()
    {
        var filtered = _allEntries.AsEnumerable();

        if (SelectedJobFilter != "Todos")
        {
            filtered = filtered.Where(e => e.Entry.JobName == SelectedJobFilter);
        }

        if (SelectedStatusFilter != "Todos")
        {
            filtered = filtered.Where(e => e.Entry.Status.Contains(SelectedStatusFilter, StringComparison.OrdinalIgnoreCase));
        }

        if (!string.IsNullOrWhiteSpace(SearchText))
        {
            filtered = filtered.Where(e => 
                e.Entry.SourcePath.Contains(SearchText, StringComparison.OrdinalIgnoreCase) ||
                e.Entry.TargetPath.Contains(SearchText, StringComparison.OrdinalIgnoreCase) ||
                e.Entry.Details.Contains(SearchText, StringComparison.OrdinalIgnoreCase)
            );
        }

        Logs.Clear();
        foreach (var item in filtered)
        {
            Logs.Add(item);
        }
    }

    [RelayCommand]
    private void ClearLogs()
    {
        try
        {
            if (Directory.Exists(_reportsFolder))
            {
                var files = Directory.GetFiles(_reportsFolder, "*.csv");
                foreach (var file in files) File.Delete(file);
            }
            
            // Também apaga o log de texto antigo por garantia
            var oldLog = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Data", "app.log");
            if (File.Exists(oldLog)) File.Delete(oldLog);

            _allEntries.Clear();
            Logs.Clear();
        }
        catch { }
    }
}
