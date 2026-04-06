using System;
using System.IO;
using System.Diagnostics;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using AutoFlow.Domain.Entities;
using AutoFlow.Application.Interfaces;

namespace AutoFlow.App.ViewModels;

public partial class AuditEntryViewModel : ViewModelBase
{
    private readonly ILocalizationService? _localizationService;
    public AuditEntry Entry { get; }

    public AuditEntryViewModel(AuditEntry entry)
    {
        Entry = entry;
        _localizationService = App.Services?.GetService(typeof(ILocalizationService)) as ILocalizationService;
    }

    [RelayCommand]
    public void OpenSource() => OpenPath(Entry.SourcePath);

    [RelayCommand]
    public void OpenTarget() => OpenPath(Entry.TargetPath);

    private void OpenPath(string? path)
    {
        if (string.IsNullOrEmpty(path)) return;
        try
        {
            var dir = Path.GetDirectoryName(path);
            if (Directory.Exists(dir))
            {
                Process.Start("explorer.exe", dir);
            }
            else if (Directory.Exists(path))
            {
                Process.Start("explorer.exe", path);
            }
        }
        catch { }
    }

    public string LocalizedStatus
    {
        get
        {
            if (_localizationService == null) return Entry.Status;
            
            return Entry.Status.ToUpper() switch
            {
                "COPIADO" => _localizationService["Copied"],
                "MOVIDO" => _localizationService["Moved"],
                "IGNORADO" => _localizationService["Ignored"],
                "FALHA" => _localizationService["FailedStatus"],
                "FALHA CRÍTICA" => _localizationService["FailedStatus"],
                "CANCELADO" => _localizationService["Cancelled"],
                "ZIPADO" => _localizationService["Zipped"],
                _ => Entry.Status
            };
        }
    }

    public string StatusColor
    {
        get
        {
            return Entry.Status.ToUpper() switch
            {
                "COPIADO" => "#10B981", // Verde
                "MOVIDO" => "#10B981",  // Verde
                "IGNORADO" => "#F59E0B", // Laranja/Amarelo
                "FALHA" => "#EF4444",   // Vermelho
                "FALHA CRÍTICA" => "#EF4444", // Vermelho
                "CANCELADO" => "#64748B", // Cinza
                _ => "{DynamicResource SystemControlForegroundBaseHighBrush}"
            };
        }
    }
}
