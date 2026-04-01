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
    private readonly DispatcherTimer _timer;

    [ObservableProperty] private ObservableCollection<JobItemViewModel> _allJobs = new();
    [ObservableProperty] private ObservableCollection<JobProgressInfo> _activeExecutions = new();
    [ObservableProperty] private bool _isQueuePaused;
    [ObservableProperty] private string _searchText = string.Empty;
    [ObservableProperty] private string _selectedFilter = "Todos";

    public ObservableCollection<string> Filters { get; } = new(new[] { "Todos", "Ativos", "Pendentes", "Watch Folder", "Cpia Direta" });

    public AutomationViewModel(
        JobAppService jobAppService,
        IJobQueue jobQueue,
        QueueProcessor queueProcessor,
        GlobalProgressService globalProgressService,
        INotificationService notificationService)
    {
        _jobAppService = jobAppService;
        _jobQueue = jobQueue;
        _queueProcessor = queueProcessor;
        _globalProgressService = globalProgressService;
        _notificationService = notificationService;

        _timer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(1) };
        _timer.Tick += (s, e) => RefreshAutomationState();
        _timer.Start();

        IsQueuePaused = _jobQueue.IsPaused;
        _ = LoadJobsAsync();
    }

    [RelayCommand]
    public async Task LoadJobsAsync()
    {
        var jobs = await _jobAppService.GetAllJobsAsync();
        var vms = jobs.Select(j => new JobItemViewModel(j, _jobAppService, _notificationService)).ToList();
        
        AllJobs.Clear();
        foreach (var vm in vms) AllJobs.Add(vm);
        
        RefreshAutomationState();
    }

    private void RefreshAutomationState()
    {
        // 1. Atualiza lista de execues ativas (The Stage)
        var active = _globalProgressService.GetActiveJobs().ToList();
        ActiveExecutions.Clear();
        foreach (var job in active) ActiveExecutions.Add(job);

        // 2. Atualiza estado de cada item na lista (Contextual UI)
        var pendingIds = _jobQueue.PendingJobs.Select(j => j.Id).ToList();
        
        foreach (var jobVm in AllJobs)
        {
            var isActive = active.Any(a => a.JobId == jobVm.Job.Id);
            var isPending = pendingIds.Contains(jobVm.Job.Id);
            
            // Aqui poderamos setar propriedades extras no JobItemViewModel para mudar o visual
            // Por enquanto, o refresh garante que os bindings de comando funcionem
        }

        IsQueuePaused = _jobQueue.IsPaused;
    }

    // Comandos de Gesto
    [RelayCommand] private void CreateDirectCopy() => (App.Services?.GetService(typeof(MainWindowViewModel)) as MainWindowViewModel)?.NavigateToJobs("DirectCopy");
    [RelayCommand] private void CreateWatchFolder() => (App.Services?.GetService(typeof(MainWindowViewModel)) as MainWindowViewModel)?.NavigateToJobs("WatchFolder");

    // Comandos de Orquestrao
    [RelayCommand] private void TogglePauseQueue() { _queueProcessor.TogglePause(); IsQueuePaused = _jobQueue.IsPaused; }
    [RelayCommand] private void StopAll() => _queueProcessor.StopAll();
    
    [RelayCommand]
    private void RunNow(Guid jobId)
    {
        var jobVm = AllJobs.FirstOrDefault(j => j.Job.Id == jobId);
        if (jobVm != null)
        {
            _jobQueue.PushToTop(jobId);
            // Se no estiver ativo, fora a entrada na fila
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
