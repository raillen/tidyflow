using System;

namespace AutoFlow.Domain.Entities;

public class FileSystemItem
{
    public string Name { get; set; } = string.Empty;
    public string FullPath { get; set; } = string.Empty;
    public bool IsDirectory { get; set; }
    public long Size { get; set; }
    public DateTime ModifiedAt { get; set; }
    public string? Extension { get; set; }
    public bool IsAuthorized { get; set; } = true;
    public bool IsHidden { get; set; }
    public bool IsReadOnly { get; set; }
}