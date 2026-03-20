using CommunityToolkit.Mvvm.ComponentModel;
using FolderFlow.Domain.Entities;

namespace FolderFlow.App.ViewModels;

public partial class JobItemViewModel : ViewModelBase
{
    [ObservableProperty]
    private bool _isSelected;

    public Job Job { get; }

    public JobItemViewModel(Job job)
    {
        Job = job;
    }
}
