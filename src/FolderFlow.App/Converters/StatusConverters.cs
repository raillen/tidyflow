using System;
using System.Globalization;
using Avalonia.Data.Converters;
using Avalonia.Media;
using Material.Icons;

namespace FolderFlow.App.Converters;

public class PlayPauseConverter : IValueConverter
{
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        bool isWatching = value is bool b && b;
        return isWatching ? MaterialIconKind.Pause : MaterialIconKind.Play;
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture) => throw new NotImplementedException();
}

public class PlayPauseColorConverter : IValueConverter
{
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        bool isWatching = value is bool b && b;
        return isWatching ? Brushes.Orange : Brushes.Green;
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture) => throw new NotImplementedException();
}
