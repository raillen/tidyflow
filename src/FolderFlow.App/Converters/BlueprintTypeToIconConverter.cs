using System;
using System.Globalization;
using Avalonia.Data.Converters;
using FolderFlow.Domain.Enums;
using Material.Icons;

namespace FolderFlow.App.Converters;

public class BlueprintTypeToIconConverter : IValueConverter
{
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        if (value is BlueprintType type)
        {
            return type == BlueprintType.File ? MaterialIconKind.FileDocumentOutline : MaterialIconKind.FolderOutline;
        }
        return MaterialIconKind.HelpCircleOutline;
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture) => throw new NotImplementedException();
}
