using System;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using AutoFlow.Application.Interfaces;
using AutoFlow.Application.Services;
using AutoFlow.Domain.Entities;
using AutoFlow.Domain.Enums;

namespace AutoFlow.App.ViewModels;

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

    [ObservableProperty] private int _totalBlueprintsCount;
    [ObservableProperty] private int _activeBlueprintsCount;
    [ObservableProperty] private int _fileBlueprintsCount;
    [ObservableProperty] private int _folderBlueprintsCount;

    public BlueprintViewModel(
        BlueprintAppService blueprintService,
        ILocalizationService localizationService,
        WatchAppService watchAppService)
    {
        _blueprintService = blueprintService;
        _localizationService = localizationService;
        _watchAppService = watchAppService;

        _timer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(5) };
        _timer.Tick += (s, e) => _ = LoadBlueprintsAsync();
        _timer.Start();

        _ = LoadBlueprintsAsync();
    }

    [RelayCommand]
    public async Task LoadBlueprintsAsync()
    {
        var blueprints = await _blueprintService.GetAllBlueprintsAsync();
        var vms = blueprints.Select(b => new BlueprintItemViewModel(b, _blueprintService, _localizationService, _watchAppService)).ToList();
        
        AllBlueprints.Clear();
        FileBlueprints.Clear();
        FolderBlueprints.Clear();
        
        foreach (var vm in vms) 
        {
            AllBlueprints.Add(vm);
            if (vm.Blueprint.Type == BlueprintType.File)
                FileBlueprints.Add(vm);
            else
                FolderBlueprints.Add(vm);
        }

        TotalBlueprintsCount = AllBlueprints.Count;
        ActiveBlueprintsCount = AllBlueprints.Count(b => b.Blueprint.IsActive);
        FileBlueprintsCount = FileBlueprints.Count;
        FolderBlueprintsCount = FolderBlueprints.Count;
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
