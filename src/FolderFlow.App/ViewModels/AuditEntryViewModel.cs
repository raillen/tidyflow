using System;
using CommunityToolkit.Mvvm.ComponentModel;
using FolderFlow.Domain.Entities;
using FolderFlow.Application.Interfaces;

namespace FolderFlow.App.ViewModels;

public partial class AuditEntryViewModel : ViewModelBase
{
    private readonly ILocalizationService? _localizationService;
    public AuditEntry Entry { get; }

    public AuditEntryViewModel(AuditEntry entry)
    {
        Entry = entry;
        _localizationService = App.Services?.GetService(typeof(ILocalizationService)) as ILocalizationService;
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
