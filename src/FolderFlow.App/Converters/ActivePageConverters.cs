using System;
using System.Globalization;
using Avalonia;
using Avalonia.Data.Converters;
using Avalonia.Media;

namespace FolderFlow.App.Converters;

public class ActivePageBackgroundConverter : IValueConverter
{
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        if (value == null || parameter == null) return Brushes.Transparent;

        string targetPageName = parameter.ToString() ?? "";
        bool isActive = value.GetType().Name.Contains(targetPageName, StringComparison.OrdinalIgnoreCase);

        if (isActive)
        {
            if (Avalonia.Application.Current?.Resources.TryGetResource("AccentColor", null, out var colorObj) == true && colorObj is Color color)
            {
                return new SolidColorBrush(color) { Opacity = 0.2 }; // Opacidade para o fundo
            }
            return Brushes.LightBlue;
        }
        return Brushes.Transparent;
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture) => throw new NotImplementedException();
}

public class ActivePageForegroundConverter : IValueConverter
{
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        if (value == null || parameter == null) return Brushes.Gray;

        string targetPageName = parameter.ToString() ?? "";
        bool isActive = value.GetType().Name.Contains(targetPageName, StringComparison.OrdinalIgnoreCase);

        if (isActive)
        {
            if (Avalonia.Application.Current?.Resources.TryGetResource("AccentColor", null, out var colorObj) == true && colorObj is Color color)
            {
                return new SolidColorBrush(color); // Texto na cor de destaque quando ativo
            }
            return Brushes.White;
        }
        
        if (Avalonia.Application.Current?.Resources.TryGetResource("TextSecondary", null, out var colorObjSec) == true && colorObjSec is ISolidColorBrush brush)
        {
            return brush;
        }
        return Brushes.Gray;
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture) => throw new NotImplementedException();
}
