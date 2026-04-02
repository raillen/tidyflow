using System;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using FolderFlow.Application.Interfaces;
using FolderFlow.Application.Services;
using FolderFlow.Domain.Entities;
using FolderFlow.Domain.Enums;

namespace FolderFlow.App.ViewModels;

public partial class BlueprintEditorViewModel : ViewModelBase
{
    private readonly BlueprintAppService _blueprintService;
    private readonly ILocalizationService _localizationService;
    private readonly IStorageService _storageService;
    private readonly IOrganizationService _organizationService;
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
    [ObservableProperty] private string _renamePreview = string.Empty;
    [ObservableProperty] private string _originalNamePreview = string.Empty;
    [ObservableProperty] private double _windowWidth = 600;
    [ObservableProperty] private int _takeN = 10;
    [ObservableProperty] private int _skipN = 5;
    [ObservableProperty] private int _counterStart = 1;
    [ObservableProperty] private int _counterPadding = 1;
    [ObservableProperty] private BlueprintType _type;
    [ObservableProperty] private bool _isFolderType;
    [ObservableProperty] private bool _toolboxVisible;
    [ObservableProperty] private bool _isNameInvalid;
    [ObservableProperty] private string _validationMessage = string.Empty;
    [ObservableProperty] private string _regexHelp = string.Empty;
    [ObservableProperty] private string _stylesHelp = string.Empty;

    public ObservableCollection<TokenInfo> AvailableTokens { get; } = new();
    public ObservableCollection<PresetInfo> Presets { get; } = new();
    public ObservableCollection<RegexRecipe> RegexRecipes { get; } = new();

    public BlueprintEditorViewModel(
        BlueprintAppService blueprintService,
        ILocalizationService localizationService,
        IStorageService storageService,
        IOrganizationService organizationService)
    {
        _blueprintService = blueprintService;
        _localizationService = localizationService;
        _storageService = storageService;
        _organizationService = organizationService;

        LoadTokenData();
    }

    private void LoadTokenData()
    {
        AvailableTokens.Clear();
        // Origem
        AvailableTokens.Add(new TokenInfo("{Original}", _localizationService["TokenOriginal"], "Source", "FileAccount"));
        AvailableTokens.Add(new TokenInfo("{Parent}", _localizationService["TokenParent"], "Source", "FolderParent"));
        
        // Tempo
        AvailableTokens.Add(new TokenInfo("{Date}", _localizationService["TokenDate"], "Time", "Calendar"));
        AvailableTokens.Add(new TokenInfo("{Year}", _localizationService["TokenYear"], "Time", "CalendarClock"));
        AvailableTokens.Add(new TokenInfo("{Month}", _localizationService["TokenMonth"], "Time", "CalendarMonth"));
        AvailableTokens.Add(new TokenInfo("{Day}", _localizationService["TokenDay"], "Time", "CalendarDay"));
        
        // Identificadores
        AvailableTokens.Add(new TokenInfo("{Counter}", _localizationService["TokenCounter"], "Id", "Numeric"));
        AvailableTokens.Add(new TokenInfo("{GUID}", _localizationService["TokenGuid"], "Id", "Fingerprint"));

        Presets.Clear();
        Presets.Add(new PresetInfo(_localizationService["PresetDate"], "{Date} - {Original}", "Calendar"));
        Presets.Add(new PresetInfo(_localizationService["PresetCounter"], "{001} - {Original}", "Numeric"));
        Presets.Add(new PresetInfo(_localizationService["PresetSnake"], "{Original:snake}", "Alpha"));
        Presets.Add(new PresetInfo(_localizationService["PresetCamel"], "{Original:camel}", "Alpha"));

        RegexRecipes.Clear();
        RegexRecipes.Add(new RegexRecipe(_localizationService["RegexRemoveNumbers"], @"\d+", ""));
        RegexRecipes.Add(new RegexRecipe(_localizationService["RegexCleanSpecial"], @"[^a-zA-Z0-9\s\-_]+", ""));
        RegexRecipes.Add(new RegexRecipe(_localizationService["RegexExtractDate"], @".*?(\d{4}-\d{2}-\d{2}).*", "$1"));
        RegexRecipes.Add(new RegexRecipe(_localizationService["RegexRemoveParentheses"], @"\s*\(.*?\)", ""));
    }

    [RelayCommand] 
    private void ToggleToolbox() 
    {
        ToolboxVisible = !ToolboxVisible;
        WindowWidth = ToolboxVisible ? 1000 : 600;
        
        RegexHelp = _localizationService["RegexAdvancedHelp"];
        StylesHelp = _localizationService["StylesAdvancedHelp"];
    }

    [RelayCommand]
    private void InsertCounter()
    {
        string token = CounterPadding > 1 ? $"{{Counter:{CounterPadding}}}" : "{Counter}";
        RenameTemplate += token;
    }

    [RelayCommand]
    private void ApplyRegexRecipe(RegexRecipe recipe)
    {
        string target = RenameTemplate.Contains("{Original}") ? "{Original}" : "";
        string mod = $"{{Original:regex({recipe.Pattern}|{recipe.Replacement})}}";
        
        if (!string.IsNullOrEmpty(target))
            RenameTemplate = RenameTemplate.Replace(target, mod);
        else
            RenameTemplate += mod;
    }

    [RelayCommand]
    private void InsertSubstring(string type)
    {
        int n = type == "take" ? TakeN : SkipN;
        string mod = $"{{Original:{type}({n})}}";
        
        if (RenameTemplate.Contains("{Original}"))
            RenameTemplate = RenameTemplate.Replace("{Original}", mod);
        else
            RenameTemplate += mod;
    }

    [RelayCommand]
    private void ApplyPreset(string template) => RenameTemplate = template;

    [RelayCommand]
    private void InsertToken(string token) => RenameTemplate += token;

    [RelayCommand]
    private void ApplyModifier(string modifier)
    {
        if (string.IsNullOrEmpty(RenameTemplate)) return;

        // Se o último caractere for }, tenta inserir o modificador antes dele
        if (RenameTemplate.EndsWith("}"))
        {
            var lastTokenIdx = RenameTemplate.LastIndexOf('{');
            if (lastTokenIdx >= 0)
            {
                var token = RenameTemplate.Substring(lastTokenIdx);
                if (!token.Contains(':'))
                {
                    RenameTemplate = RenameTemplate.Insert(RenameTemplate.Length - 1, $":{modifier}");
                }
                else if (token.Contains(':'))
                {
                    // Substitui modificador existente se for do mesmo grupo (simples)
                    // Para V2, apenas concatenamos ou ignoramos por segurança
                }
            }
        }
        else
        {
            // Se não terminar com token, aplica ao {Original} se existir ou adiciona
            if (RenameTemplate.Contains("{Original}"))
                RenameTemplate = RenameTemplate.Replace("{Original}", $"{{Original:{modifier}}}");
            else
                RenameTemplate += $"{{Original:{modifier}}}";
        }
    }

    public void SetBlueprint(Blueprint blueprint)
    {
        _originalBlueprint = blueprint;
        Name = blueprint.Name;
        Path = blueprint.Path;
        Type = blueprint.Type;
        IsFolderType = Type == BlueprintType.Folder;
        RenameTemplate = blueprint.RenameTemplate ?? string.Empty;
        AutoScaffoldingEnabled = blueprint.AutoScaffoldingEnabled;
        AutoRenamingEnabled = blueprint.AutoRenamingEnabled;
        CounterStart = blueprint.CounterStart;
        CounterPadding = blueprint.CounterPadding;
        IsActive = blueprint.IsActive;
        
        Folders.Clear();
        foreach (var f in blueprint.BlueprintFolders) Folders.Add(f);
        
        UpdatePreview();
    }

    partial void OnRenameTemplateChanged(string value) => UpdatePreview();
    partial void OnNameChanged(string value) => UpdatePreview();
    partial void OnCounterStartChanged(int value) => UpdatePreview();
    partial void OnCounterPaddingChanged(int value) => UpdatePreview();

    private void UpdatePreview()
    {
        OriginalNamePreview = IsFolderType 
            ? "2024-01-05 Backup_Final #001 (Projeto Importante) @v1.2" 
            : "2024-01-05 Backup_Final #001 (Projeto Importante) @v1.2.pdf";

        if (string.IsNullOrWhiteSpace(RenameTemplate))
        {
            RenamePreview = OriginalNamePreview;
            IsNameInvalid = false;
            return;
        }

        // Simula um item para o preview (Arquivo ou Pasta)
        var dummyPath = System.IO.Path.Combine(string.IsNullOrEmpty(Path) ? "C:\\Work\\FolderFlow" : Path, OriginalNamePreview);

        _ = Task.Run(async () => {
            var result = await _organizationService.GetRenamedPathAsync(RenameTemplate, Name, dummyPath, IsFolderType, CounterStart);
            var resultName = System.IO.Path.GetFileName(result);
            
            Avalonia.Threading.Dispatcher.UIThread.Post(() => {
                RenamePreview = resultName;
                ValidateName(resultName);
            });
        });
    }

    private void ValidateName(string name)
    {
        char[] invalidChars = System.IO.Path.GetInvalidFileNameChars();
        var found = name.Where(c => invalidChars.Contains(c)).Distinct().ToList();
        
        if (found.Any())
        {
            IsNameInvalid = true;
            ValidationMessage = $"{_localizationService["InvalidChars"]}: {string.Join(" ", found)}";
        }
        else
        {
            IsNameInvalid = false;
            ValidationMessage = string.Empty;
        }
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
        _originalBlueprint.Type = Type;
        _originalBlueprint.RenameTemplate = RenameTemplate;
        _originalBlueprint.AutoScaffoldingEnabled = AutoScaffoldingEnabled;
        _originalBlueprint.AutoRenamingEnabled = AutoRenamingEnabled;
        _originalBlueprint.CounterStart = CounterStart;
        _originalBlueprint.CounterPadding = CounterPadding;
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

public record TokenInfo(string Token, string Description, string Category, string Icon);
public record PresetInfo(string Name, string Template, string Icon);
public record RegexRecipe(string Name, string Pattern, string Replacement);
