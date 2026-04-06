using System;
using System.Globalization;
using Avalonia.Data.Converters;
using Material.Icons;

namespace AutoFlow.App.Converters;

public class PageToColorConverter : IValueConverter
{
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        bool isWatchMode = value is bool b && b;
        string? param = parameter as string;

        if (param == "Icon")
        {
            return isWatchMode ? MaterialIconKind.Radar : MaterialIconKind.CalendarClockOutline;
        }
        
        if (param == "Text")
        {
            return isWatchMode ? "Hotfolder" : "Agenda";
        }

        return null;
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture) => throw new NotImplementedException();
}
