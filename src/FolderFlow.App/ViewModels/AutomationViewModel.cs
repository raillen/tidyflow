using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using FolderFlow.App.Services;
using FolderFlow.Application.Interfaces;
using FolderFlow.Application.Services;
using FolderFlow.Domain.Entities;
using FolderFlow.Domain.ValueObjects;

namespace FolderFlow.App.ViewModels;

public partial class AutomationViewModel : ViewModelBase, IDisposable
{
    private readonly JobAppService _jobAppService;
    private readonly IJobQueue _jobQueue;
    private readonly QueueProcessor _queueProcessor;
    private readonly GlobalProgressService _globalProgressService;
    private readonly INotificationService _notificationService;
    private readonly ILocalizationService _localizationService;
    private readonly DispatcherTimer _timer;

    [ObservableProperty] private ObservableCollection<JobItemViewModel> _allJobs = new();
    [ObservableProperty] private ObservableCollection<JobProgressInfo> _activeExecutions = new();
    [ObservableProperty] private bool _isQueuePaused;
    [ObservableProperty] private string _searchText = string.Empty;
    [ObservableProperty] private string _selectedFilter = "";

    partial void OnSelectedFilterChanged(string value)
    {
        // Se o valor selecionado for a traduo de "Todos", mapeamos internamente se necessrio
        // Mas o melhor  usar a prpria traduo para comparar se ela vier do combo.
        // Como o RadioButton no XAML usa EqualsConverter com string fixa 'Todos',
        // precisamos atualizar o XAML ou o ViewModel.
    }

    public ObservableCollection<string> Filters { get; } = new();

    public AutomationViewModel(
        JobAppService jobAppService,
        IJobQueue jobQueue,
        QueueProcessor queueProcessor,
        GlobalProgressService globalProgressService,
        INotificationService notificationService,
        ILocalizationService localizationService)
    {
        _jobAppService = jobAppService;
        _jobQueue = jobQueue;
        _queueProcessor = queueProcessor;
        _globalProgressService = globalProgressService;
        _notificationService = notificationService;
        _localizationService = localizationService;

        _selectedFilter = _localizationService["All"];
        UpdateFilters();

        _timer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(1) };
        _timer.Tick += (s, e) => RefreshAutomationState();
        _timer.Start();

        if (_localizationService is System.ComponentModel.INotifyPropertyChanged npc)
        {
            npc.PropertyChanged += (s, e) => {
                if (e.PropertyName == "Item" || e.PropertyName == "Item[]" || string.IsNullOrEmpty(e.PropertyName))
                {
                    UpdateFilters();
                    UpdateSelectionState();
                    RefreshAutomationState();
                }
            };
        }

        IsQueuePaused = _jobQueue.IsPaused;
        _ = LoadJobsAsync();
    }

    private void UpdateFilters()
    {
        var current = SelectedFilter;
        Filters.Clear();
        Filters.Add("All");
        Filters.Add("Active");
        Filters.Add("Pending");
        Filters.Add("WatchFolder");
        Filters.Add("DirectCopy");
        SelectedFilter = Filters.Contains(current) ? current : "All";
    }

    [RelayCommand]
    public async Task LoadJobsAsync()
    {
        var jobs = await _jobAppService.GetAllJobsAsync();
        var vms = jobs.Select(j => new JobItemViewModel(j, _jobAppService, _notificationService, _localizationService)).ToList();
        
        AllJobs.Clear();
        foreach (var vm in vms) AllJobs.Add(vm);
        
        RefreshAutomationState();
    }

    [ObservableProperty] private bool _isSelectedAll;
    [ObservableProperty] private bool _hasSelection;
    [ObservableProperty] private int _selectedCount;
    [ObservableProperty] private string _selectedCountText = string.Empty;
    [ObservableProperty] private string _activeBadgeText = string.Empty;

    public void UpdateSelectionState()
    {
        SelectedCount = AllJobs.Count(j => j.IsSelected);
        HasSelection = SelectedCount > 0;
        SelectedCountText = string.Format(_localizationService["SelectedCount"], SelectedCount);
        
        var shouldBeAll = AllJobs.Any() && SelectedCount == AllJobs.Count;
        if (IsSelectedAll != shouldBeAll)
        {
            IsSelectedAll = shouldBeAll;
        }
    }

    private void RefreshAutomationState()
    {
        var active = _globalProgressService.GetActiveJobs().ToList();
        ActiveExecutions.Clear();
        foreach (var job in active) ActiveExecutions.Add(job);
        
        ActiveBadgeText = string.Format(_localizationService["ActiveBadge"], ActiveExecutions.Count);

        foreach (var jobVm in AllJobs)
        {
            var p = active.FirstOrDefault(a => a.JobId == jobVm.Job.Id);
            if (p != null) jobVm.UpdateFromProgress(p);
            else jobVm.Status = _queueProcessor.IsJobActive(jobVm.Job.Id) ? _localizationService["Processing"] : _localizationService["Idle"];
        }

        UpdateSelectionState();
        IsQueuePaused = _jobQueue.IsPaused;
    }

    [RelayCommand]
    private void ToggleSelectAll()
    {
        foreach (var job in AllJobs) job.IsSelected = IsSelectedAll;
        UpdateSelectionState();
    }

    [RelayCommand]
    private async Task DeleteSelected()
    {
        var selected = AllJobs.Where(j => j.IsSelected).ToList();
        if (!selected.Any()) return;

        foreach (var jobVm in selected)
        {
            await _jobAppService.DeleteJobAsync(jobVm.Job.Id);
            AllJobs.Remove(jobVm);
        }
        UpdateSelectionState();
    }

    [RelayCommand]
    private async Task RunSelected()
    {
        var selected = AllJobs.Where(j => j.IsSelected).ToList();
        foreach (var jobVm in selected) await jobVm.RunNowCommand.ExecuteAsync(null);
    }

    [RelayCommand]
    private void StopSelected()
    {
        var selected = AllJobs.Where(j => j.IsSelected).ToList();
        foreach (var jobVm in selected) _queueProcessor.StopJob(jobVm.Job.Id);
    }

    // Comandos de Gesto - Corrigidos para evitar loop
    [RelayCommand] 
    private void CreateDirectCopy() 
    {
        var mainVm = App.Services?.GetService(typeof(MainWindowViewModel)) as MainWindowViewModel;
        mainVm?.ShowEditor(new Job { WatchEnabled = false, Name = _localizationService["NewDirectCopy"] });
    }

    [RelayCommand] 
    private void CreateWatchFolder() 
    {
        var mainVm = App.Services?.GetService(typeof(MainWindowViewModel)) as MainWindowViewModel;
        mainVm?.ShowEditor(new Job { WatchEnabled = true, Name = _localizationService["NewWatchFolder"] });
    }

    // Comandos de Orquestrao
    [RelayCommand] private void TogglePauseQueue() { _queueProcessor.TogglePause(); IsQueuePaused = _jobQueue.IsPaused; }
    [RelayCommand] private void StopAll() => _queueProcessor.StopAll();
    
    [RelayCommand]
    private void StopJob(Guid jobId) => _queueProcessor.StopJob(jobId);

    [RelayCommand]
    private void RunNow(Guid jobId)
    {
        var jobVm = AllJobs.FirstOrDefault(j => j.Job.Id == jobId);
        if (jobVm != null)
        {
            _jobQueue.PushToTop(jobId);
            if (!_queueProcessor.IsJobActive(jobId)) jobVm.RunNowCommand.Execute(null);
        }
    }

    [RelayCommand]
    private async Task DeleteJob(Guid jobId)
    {
        var jobVm = AllJobs.FirstOrDefault(j => j.Job.Id == jobId);
        if (jobVm != null)
        {
            await _jobAppService.DeleteJobAsync(jobId);
            AllJobs.Remove(jobVm);
        }
    }

    public void Dispose() => _timer.Stop();
}
