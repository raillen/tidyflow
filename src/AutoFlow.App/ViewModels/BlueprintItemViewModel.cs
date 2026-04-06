using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using AutoFlow.Application.Interfaces;
using AutoFlow.Application.Services;
using AutoFlow.Domain.Entities;

namespace AutoFlow.App.ViewModels;

public partial class BlueprintItemViewModel : ViewModelBase
{
    private readonly BlueprintAppService _blueprintService;
    private readonly ILocalizationService _localizationService;
    private readonly WatchAppService _watchAppService;

    [ObservableProperty] private Blueprint _blueprint;

    public BlueprintItemViewModel(Blueprint blueprint, BlueprintAppService blueprintService, ILocalizationService localizationService, WatchAppService watchAppService)
    {
        _blueprint = blueprint;
        _blueprintService = blueprintService;
        _localizationService = localizationService;
        _watchAppService = watchAppService;
    }

    [RelayCommand]
    private void Edit()
    {
        var mainVm = App.Services?.GetService(typeof(MainWindowViewModel)) as MainWindowViewModel;
        mainVm?.ShowBlueprintEditor(Blueprint);
    }

    [RelayCommand]
    private async Task Delete()
    {
        await _blueprintService.DeleteBlueprintAsync(Blueprint.Id);
        var mainVm = App.Services?.GetService(typeof(MainWindowViewModel)) as MainWindowViewModel;
        mainVm?.NavigateToBlueprint();
    }

    [RelayCommand]
    private async Task ToggleActive()
    {
        Blueprint.IsActive = !Blueprint.IsActive;
        await _blueprintService.SaveBlueprintAsync(Blueprint);
        _watchAppService.UpdateBlueprintWatching(Blueprint);
    }
}
