using System;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
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
    private readonly IStorageService _storageService;
    private readonly IOrganizationService _organizationService;
    private readonly ILocalizationService _localizationService;

    [ObservableProperty] private Blueprint _blueprint = new();
    [ObservableProperty] private string _pathText = string.Empty;
    [ObservableProperty] private ObservableCollection<string> _blueprintFolders = new();
    [ObservableProperty] private string _newFolderName = string.Empty;
    [ObservableProperty] private string _renameTemplateText = string.Empty;
    [ObservableProperty] private string _renamePreviewResult = string.Empty;

    public event Action? Saved;
    public event Action? Cancelled;

    public BlueprintEditorViewModel(
        BlueprintAppService blueprintService,
        IStorageService storageService,
        IOrganizationService organizationService,
        ILocalizationService localizationService)
    {
        _blueprintService = blueprintService;
        _storageService = storageService;
        _organizationService = organizationService;
        _localizationService = localizationService;
    }

    public void SetBlueprint(Blueprint blueprint)
    {
        Blueprint = blueprint;
        PathText = blueprint.Path;
        BlueprintFolders = new ObservableCollection<string>(blueprint.BlueprintFolders);
        RenameTemplateText = blueprint.RenameTemplate ?? "";
        RenamePreviewResult = string.Empty;
    }

    [RelayCommand]
    private async Task SelectPath()
    {
        var path = await _storageService.SelectFolderAsync();
        if (path != null) PathText = path;
    }

    [RelayCommand]
    private void AddFolder()
    {
        if (!string.IsNullOrWhiteSpace(NewFolderName))
        {
            if (!BlueprintFolders.Contains(NewFolderName)) BlueprintFolders.Add(NewFolderName);
            NewFolderName = string.Empty;
        }
    }

    [RelayCommand]
    private void RemoveFolder(string folderName) => BlueprintFolders.Remove(folderName);

    [RelayCommand]
    private async Task TestRename()
    {
        if (string.IsNullOrWhiteSpace(RenameTemplateText)) return;
        var mockPath = System.IO.Path.Combine(PathText, "file_sample.mp4");
        RenamePreviewResult = await _organizationService.GetRenamedPathAsync(RenameTemplateText, Blueprint.Name, mockPath);
        RenamePreviewResult = System.IO.Path.GetFileName(RenamePreviewResult);
    }

    [RelayCommand]
    private async Task Save()
    {
        Blueprint.Path = PathText;
        Blueprint.BlueprintFolders = BlueprintFolders.ToList();
        Blueprint.RenameTemplate = RenameTemplateText;
        await _blueprintService.SaveBlueprintAsync(Blueprint);
        Saved?.Invoke();
    }

    [RelayCommand]
    private void Cancel() => Cancelled?.Invoke();
}
