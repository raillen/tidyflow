using System;
using System.Collections.ObjectModel;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using FolderFlow.Application.Interfaces;
using FolderFlow.Application.Services;
using FolderFlow.Domain.Entities;

namespace FolderFlow.App.ViewModels;

public partial class BlueprintEditorViewModel : ViewModelBase
{
    private readonly BlueprintAppService _blueprintService;
    private readonly ILocalizationService _localizationService;
    private readonly IStorageService _storageService;
    private Blueprint _originalBlueprint = null!;

    public event Action? Saved;
    public event Action? Cancelled;

    [ObservableProperty] private string _name = string.Empty;
    [ObservableProperty] private string _path = string.Empty;
    [ObservableProperty] private string _renameTemplate = string.Empty;
    [ObservableProperty] private bool _autoScaffoldingEnabled;
    [ObservableProperty] private bool _autoRenamingEnabled;
    [ObservableProperty] private bool _isActive = true;
    [ObservableProperty] private ObservableCollection<string> _folders = new();
    [ObservableProperty] private string _newFolderName = string.Empty;

    public BlueprintEditorViewModel(
        BlueprintAppService blueprintService,
        ILocalizationService localizationService,
        IStorageService storageService)
    {
        _blueprintService = blueprintService;
        _localizationService = localizationService;
        _storageService = storageService;
    }

    public void SetBlueprint(Blueprint blueprint)
    {
        _originalBlueprint = blueprint;
        Name = blueprint.Name;
        Path = blueprint.Path;
        RenameTemplate = blueprint.RenameTemplate ?? string.Empty;
        AutoScaffoldingEnabled = blueprint.AutoScaffoldingEnabled;
        AutoRenamingEnabled = blueprint.AutoRenamingEnabled;
        IsActive = blueprint.IsActive;
        
        Folders.Clear();
        foreach (var f in blueprint.BlueprintFolders) Folders.Add(f);
    }

    [RelayCommand]
    private async Task BrowsePath()
    {
        var result = await _storageService.SelectFolderAsync();
        if (result != null) Path = result;
    }

    [RelayCommand]
    private void AddFolder()
    {
        if (!string.IsNullOrWhiteSpace(NewFolderName) && !Folders.Contains(NewFolderName))
        {
            Folders.Add(NewFolderName);
            NewFolderName = string.Empty;
        }
    }

    [RelayCommand]
    private void RemoveFolder(string folder) => Folders.Remove(folder);

    [RelayCommand]
    private async Task Save()
    {
        _originalBlueprint.Name = Name;
        _originalBlueprint.Path = Path;
        _originalBlueprint.RenameTemplate = RenameTemplate;
        _originalBlueprint.AutoScaffoldingEnabled = AutoScaffoldingEnabled;
        _originalBlueprint.AutoRenamingEnabled = AutoRenamingEnabled;
        _originalBlueprint.IsActive = IsActive;
        _originalBlueprint.BlueprintFolders = new System.Collections.Generic.List<string>(Folders);

        await _blueprintService.SaveBlueprintAsync(_originalBlueprint);
        
        // Notifica o Watcher
        var watchService = App.Services?.GetService(typeof(WatchAppService)) as WatchAppService;
        watchService?.UpdateBlueprintWatching(_originalBlueprint);

        Saved?.Invoke();
    }

    [RelayCommand]
    private void Cancel() => Cancelled?.Invoke();
}
