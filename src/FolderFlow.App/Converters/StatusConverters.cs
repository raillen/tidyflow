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

public class StatusToProgressColorConverter : IValueConverter
{
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        var status = value?.ToString()?.ToUpper() ?? "";
        if (status.Contains("FALHA")) return Brushes.Red;
        if (status.Contains("IGNORADO") || status.Contains("SKIP")) return Brushes.Yellow;
        if (status.Contains("CANCELADO")) return Brushes.Gray;
        return Brushes.DeepSkyBlue;
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture) => throw new NotImplementedException();
}

public class BoolToIconConverter : IValueConverter
{
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        bool b = value is bool && (bool)value;
        return b ? MaterialIconKind.ChevronUp : MaterialIconKind.ChevronDown;
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture) => throw new NotImplementedException();
}

public class StatusToPercentageConverter : IValueConverter
{
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        var status = value?.ToString()?.ToUpper() ?? "";
        if (status.Contains("IGNORADO") || status.Contains("SKIP") || status.Contains("COPIADO") || status.Contains("MOVIDO"))
            return 100.0;
        return Avalonia.AvaloniaProperty.UnsetValue; // Let the original percentage be used
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture) => throw new NotImplementedException();
}
