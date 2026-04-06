using System;
using System.Globalization;
using Avalonia.Data.Converters;
using Avalonia.Media;
using Material.Icons;

namespace AutoFlow.App.Converters;

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
        string paramStr = parameter?.ToString() ?? "";

        // Suporte para "TrueIcon:FalseIcon"
        if (paramStr.Contains(':'))
        {
            var parts = paramStr.Split(':');
            if (parts.Length == 2)
            {
                var iconName = b ? parts[0] : parts[1];
                if (Enum.TryParse<MaterialIconKind>(iconName, out var kind)) return kind;
            }
        }

        return b ? MaterialIconKind.ChevronUp : MaterialIconKind.ChevronDown;
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture) => throw new NotImplementedException();
}

public class BoolToTextConverter : IValueConverter
{
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        bool b = value is bool && (bool)value;
        string paramStr = parameter?.ToString() ?? "";

        if (paramStr.Contains(':'))
        {
            var parts = paramStr.Split(':');
            if (parts.Length == 2)
            {
                return b ? parts[0] : parts[1];
            }
        }

        return b.ToString();
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture) => throw new NotImplementedException();
}

public class BoolToBrushConverter : IValueConverter
{
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        bool b = value is bool && (bool)value;
        string paramStr = parameter?.ToString() ?? "";

        if (paramStr.Contains(':'))
        {
            var parts = paramStr.Split(':');
            if (parts.Length == 2)
            {
                // Tenta resolver como recurso dinâmico ou cor fixa
                var colorName = b ? parts[0] : parts[1];
                
                if (colorName == "DangerColor") return Brushes.Red;
                if (colorName == "AccentColor") return Brushes.DeepSkyBlue;
                
                return b ? Brushes.Red : Brushes.Transparent;
            }
        }

        return b ? Brushes.Green : Brushes.Red;
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
