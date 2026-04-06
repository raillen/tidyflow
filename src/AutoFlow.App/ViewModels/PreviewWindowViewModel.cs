using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using AutoFlow.Domain.ValueObjects;
using AutoFlow.Application.Interfaces;

namespace AutoFlow.App.ViewModels;

public partial class PreviewWindowViewModel : ViewModelBase
{
    private readonly ILocalizationService _localizationService;

    [ObservableProperty] private PreviewSummary _summary;
    
    public ObservableCollection<string> AffectedPaths { get; } = new();

    public PreviewWindowViewModel(PreviewSummary summary, ILocalizationService localizationService)
    {
        _summary = summary;
        _localizationService = localizationService;
        
        foreach (var path in summary.AffectedPaths)
        {
            AffectedPaths.Add(path);
        }
    }
}