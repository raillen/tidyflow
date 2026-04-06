using System.Collections.Generic;

namespace AutoFlow.Domain.ValueObjects;

public class PreviewSummary
{
    public int FilesToCopy { get; set; }
    public int FilesToMove { get; set; }
    public int FilesToSkip { get; set; }
    public int FilesToOverwrite { get; set; }
    public long TotalBytesToTransfer { get; set; }
    
    // Helper Properties for Premium Dashboard/Editor
    public int TotalFiles => FilesToCopy + FilesToMove;
    public long TotalBytes => TotalBytesToTransfer;

    public List<string> AffectedPaths { get; set; } = new();
}
