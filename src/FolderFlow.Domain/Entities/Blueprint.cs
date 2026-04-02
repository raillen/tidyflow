using System;
using System.Collections.Generic;

namespace FolderFlow.Domain.Entities;

public class Blueprint
{
    public Guid Id { get; set; } = Guid.NewGuid();
    public string Name { get; set; } = string.Empty;
    public string Path { get; set; } = string.Empty;
    
    // Scaffolding: Pastas a serem criadas automaticamente
    public List<string> BlueprintFolders { get; set; } = new();
    
    // Renaming: Template para renomeação de novos arquivos
    public string? RenameTemplate { get; set; }
    
    public bool IsActive { get; set; } = true;
    public DateTime CreatedAt { get; set; } = DateTime.Now;
}
