using System;
using CommunityToolkit.Mvvm.ComponentModel;
using FolderFlow.Domain.Entities;

namespace FolderFlow.App.ViewModels;

public partial class AuditEntryViewModel : ViewModelBase
{
    public AuditEntry Entry { get; }

    public AuditEntryViewModel(AuditEntry entry)
    {
        Entry = entry;
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
