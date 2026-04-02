using System;
using System.Globalization;
using System.Linq;
using Avalonia;
using Avalonia.Data.Converters;

namespace FolderFlow.App.Converters;

public class EqualsConverter : IValueConverter
{
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        if (value == null || parameter == null) return false;

        var paramStr = parameter.ToString() ?? "";
        
        // Suporte para traduo condicional: "True:TextoA;False:TextoB"
        if (paramStr.Contains(':') && paramStr.Contains(';'))
        {
            var valStr = value.ToString() ?? "";
            var pairs = paramStr.Split(';');
            foreach (var pair in pairs)
            {
                var parts = pair.Split(':');
                if (parts.Length == 2 && parts[0].Trim().Equals(valStr, StringComparison.OrdinalIgnoreCase))
                {
                    return parts[1].Trim();
                }
            }
        }

        return value.ToString() == paramStr;
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        if (value is bool b && b) return parameter;
        return AvaloniaProperty.UnsetValue;
    }
}

public class NotEqualsConverter : IValueConverter
{
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        if (value == null || parameter == null) return true;
        return value.ToString() != parameter.ToString();
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        if (value is bool b && b) return parameter;
        return AvaloniaProperty.UnsetValue;
    }
}
