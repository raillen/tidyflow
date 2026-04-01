using System;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using FolderFlow.Application.Interfaces;
using FolderFlow.Application.Services;
using FolderFlow.Domain.Entities;

namespace FolderFlow.App.ViewModels;

public partial class JobItemViewModel : ViewModelBase
{
    private readonly JobAppService _jobAppService;
    private readonly INotificationService _notificationService;

    [ObservableProperty] private bool _isSelected;
    [ObservableProperty] private string _status = "Ocioso";
    [ObservableProperty] private double _progress;

    public Job Job { get; }

    public JobItemViewModel(Job job, JobAppService jobAppService, INotificationService notificationService)
    {
        Job = job;
        _jobAppService = jobAppService;
        _notificationService = notificationService;
    }

    [RelayCommand]
    public async Task RunNow()
    {
        Status = "Iniciando...";
        await _jobAppService.RunJobAsync(Job.Id);
    }

    [RelayCommand]
    public void Edit()
    {
        (App.Services?.GetService(typeof(MainWindowViewModel)) as MainWindowViewModel)?.NavigateToJobs("Edit");
        // O EditorViewModel deve ser carregado com este Job
    }
}
