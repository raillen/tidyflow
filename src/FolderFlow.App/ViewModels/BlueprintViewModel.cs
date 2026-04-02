using System;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using FolderFlow.Application.Interfaces;
using FolderFlow.Application.Services;
using FolderFlow.Domain.Entities;
using FolderFlow.Domain.Enums;

namespace FolderFlow.App.ViewModels;

public partial class BlueprintViewModel : ViewModelBase, IDisposable
{
    private readonly BlueprintAppService _blueprintService;
    private readonly ILocalizationService _localizationService;
    private readonly WatchAppService _watchAppService;
    private readonly DispatcherTimer _timer;

    [ObservableProperty] private ObservableCollection<BlueprintItemViewModel> _allBlueprints = new();
    [ObservableProperty] private ObservableCollection<BlueprintItemViewModel> _fileBlueprints = new();
    [ObservableProperty] private ObservableCollection<BlueprintItemViewModel> _folderBlueprints = new();
    [ObservableProperty] private string _searchText = string.Empty;

    public BlueprintViewModel(
        BlueprintAppService blueprintService,
        ILocalizationService localizationService,
        WatchAppService watchAppService)
    {
        _blueprintService = blueprintService;
        _localizationService = localizationService;
        _watchAppService = watchAppService;

        _timer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(2) };
        _timer.Tick += (s, e) => _ = LoadBlueprintsAsync();
        _timer.Start();

        _ = LoadBlueprintsAsync();
    }

    [RelayCommand]
    public async Task LoadBlueprintsAsync()
    {
        var blueprints = await _blueprintService.GetAllBlueprintsAsync();
        
        // Simples refresh por enquanto
        var vms = blueprints.Select(b => new BlueprintItemViewModel(b, _blueprintService, _localizationService, _watchAppService)).ToList();
        
        AllBlueprints.Clear();
        FileBlueprints.Clear();
        FolderBlueprints.Clear();

        foreach (var vm in vms) 
        {
            AllBlueprints.Add(vm);
            if (vm.Blueprint.Type == FolderFlow.Domain.Enums.BlueprintType.File)
                FileBlueprints.Add(vm);
            else
                FolderBlueprints.Add(vm);
        }
    }

    [RelayCommand]
    private void CreateFileBlueprint()
    {
        var mainVm = App.Services?.GetService(typeof(MainWindowViewModel)) as MainWindowViewModel;
        mainVm?.ShowBlueprintEditor(new Blueprint 
        { 
            Name = _localizationService["NewFileBlueprintTitle"],
            Type = BlueprintType.File
        });
    }

    [RelayCommand]
    private void CreateFolderBlueprint()
    {
        var mainVm = App.Services?.GetService(typeof(MainWindowViewModel)) as MainWindowViewModel;
        mainVm?.ShowBlueprintEditor(new Blueprint 
        { 
            Name = _localizationService["NewFolderBlueprintTitle"],
            Type = BlueprintType.Folder
        });
    }

    public void Dispose() => _timer.Stop();
}

public partial class BlueprintItemViewModel : ViewModelBase
{
    private readonly BlueprintAppService _blueprintService;
    private readonly ILocalizationService _localizationService;
    private readonly WatchAppService _watchAppService;

    public Blueprint Blueprint { get; }
    public string TypeLabel => Blueprint.Type == BlueprintType.File ? "FILE" : "FOLDER";
    public string IconPath => Blueprint.Type == BlueprintType.File ? "file_regular" : "folder_regular";

    public BlueprintItemViewModel(Blueprint blueprint, BlueprintAppService blueprintService, ILocalizationService localizationService, WatchAppService watchAppService)
    {
        Blueprint = blueprint;
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
        _ = mainVm?.Blueprint.LoadBlueprintsAsync();
    }

    [RelayCommand]
    private async Task ToggleActive()
    {
        Blueprint.IsActive = !Blueprint.IsActive;
        await _blueprintService.SaveBlueprintAsync(Blueprint);
        _watchAppService.UpdateBlueprintWatching(Blueprint);
    }
}
