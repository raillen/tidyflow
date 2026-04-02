using System;
using System.Collections.Generic;
using FolderFlow.Domain.Enums;

namespace FolderFlow.Domain.Entities;

public class Blueprint
{
    public Guid Id { get; set; } = Guid.NewGuid();
    public string Name { get; set; } = string.Empty;
    public string Path { get; set; } = string.Empty;
    public BlueprintType Type { get; set; } = BlueprintType.File;
    
    // Scaffolding: Pastas a serem criadas automaticamente (Para Tipo Folder)
    public List<string> BlueprintFolders { get; set; } = new();
    public bool AutoScaffoldingEnabled { get; set; } = false;
    
    // Renaming: Template para renomeação de novos arquivos ou pastas
    public string? RenameTemplate { get; set; }
    public bool AutoRenamingEnabled { get; set; } = false;
    public int CounterStart { get; set; } = 1;
    
    public bool IsActive { get; set; } = true;
    public DateTime CreatedAt { get; set; } = DateTime.Now;
}
