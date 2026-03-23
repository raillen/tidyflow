using System;
using System.Globalization;
using Avalonia.Data.Converters;
using Avalonia.Media;

namespace FolderFlow.App.Converters;

public class PageToColorConverter : IValueConverter
{
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        bool isActive = value is bool b && b;
        return isActive ? Brush.Parse("#A8558D") : Brush.Parse("#64748B");
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        throw new NotImplementedException();
    }
}
