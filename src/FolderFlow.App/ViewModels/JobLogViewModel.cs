using CommunityToolkit.Mvvm.ComponentModel;
using FolderFlow.Domain.ValueObjects;

namespace FolderFlow.App.ViewModels;

public partial class JobLogViewModel : ViewModelBase
{
    [ObservableProperty]
    private JobProgressInfo _progress;

    [ObservableProperty]
    private bool _isExpanded;

    public JobLogViewModel(JobProgressInfo progress)
    {
        _progress = progress;
    }

    public void Update(JobProgressInfo p)
    {
        Progress = p;
        OnPropertyChanged(nameof(Progress));
    }
}
