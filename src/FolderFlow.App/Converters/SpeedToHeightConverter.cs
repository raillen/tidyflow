using System;
using System.Globalization;
using Avalonia.Data.Converters;

namespace FolderFlow.App.Converters;

public class SpeedToHeightConverter : IValueConverter
{
    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        if (value is double speed)
        {
            // Escala: 100 MB/s = 60px de altura
            double maxSpeed = 100.0 * 1024 * 1024; 
            double height = (speed / maxSpeed) * 60.0;
            return Math.Clamp(height, 2, 60); // Mnimo 2px para aparecer algo
        }
        return 2.0;
    }

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        throw new NotImplementedException();
    }
}
