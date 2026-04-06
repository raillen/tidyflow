using System;
using System.IO;
using System.Linq;
using AutoFlow.Application.Interfaces;
using MetadataExtractor;
using MetadataExtractor.Formats.Exif;

namespace AutoFlow.Infrastructure.Filesystem;

public class MetadataService : IMetadataService
{
    public DateTime? GetExifDate(string filePath)
    {
        try
        {
            var ext = Path.GetExtension(filePath).ToLowerInvariant();
            if (ext != ".jpg" && ext != ".jpeg" && ext != ".png" && ext != ".tiff") return null;

            var directories = ImageMetadataReader.ReadMetadata(filePath);
            var subIfdDirectory = directories.OfType<ExifSubIfdDirectory>().FirstOrDefault();

            if (subIfdDirectory != null && subIfdDirectory.TryGetDateTime(ExifDirectoryBase.TagDateTimeOriginal, out var date))
            {
                return date;
            }
        }
        catch
        {
            // Ignore errors like file in use or not an image
        }
        return null;
    }

    public bool ContainsText(string filePath, string searchText)
    {
        if (string.IsNullOrWhiteSpace(searchText)) return false;

        try
        {
            var ext = Path.GetExtension(filePath).ToLowerInvariant();
            // Somente ler arquivos de texto conhecidos para não travar lendo gigabytes binários
            if (ext != ".txt" && ext != ".md" && ext != ".csv" && ext != ".json" && ext != ".xml" && ext != ".log") return false;

            // Lendo de forma leve e progressiva
            using var stream = new FileStream(filePath, FileMode.Open, FileAccess.Read, FileShare.ReadWrite);
            using var reader = new StreamReader(stream);
            
            string? line;
            while ((line = reader.ReadLine()) != null)
            {
                if (line.Contains(searchText, StringComparison.OrdinalIgnoreCase))
                {
                    return true;
                }
            }
        }
        catch
        {
            // Ignore if file is locked or cannot be read
        }
        
        return false;
    }
}
