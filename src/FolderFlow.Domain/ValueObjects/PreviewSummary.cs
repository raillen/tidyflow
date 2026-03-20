using System.Collections.Generic;

namespace FolderFlow.Domain.ValueObjects;

public class PreviewSummary
{
    public int FilesToCopy { get; set; }
    public int FilesToMove { get; set; }
    public int FilesToSkip { get; set; }
    public int FilesToOverwrite { get; set; }
    public long TotalBytesToTransfer { get; set; }
    
    public List<string> AffectedPaths { get; set; } = new();
}
