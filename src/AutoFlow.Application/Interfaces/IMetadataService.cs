using System;

namespace AutoFlow.Application.Interfaces;

public interface IMetadataService
{
    DateTime? GetExifDate(string filePath);
    bool ContainsText(string filePath, string searchText);
}
