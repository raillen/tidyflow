using System;
using System.Globalization;
using Avalonia.Data.Converters;
using FolderFlow.Application.Interfaces;
using Microsoft.Extensions.DependencyInjection;

namespace FolderFlow.App.Converters;

public class EnumLocalizationConverter : IValueConverter
{
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        if (value == null) return null;

        var localizationService = App.Services?.GetService<ILocalizationService>();
        if (localizationService == null) return value.ToString();

        string key = value.ToString() ?? string.Empty;
        
        // Algumas chaves de Enum no Job podem precisar de mapeamento se os nomes forem diferentes
        // Por exemplo, ScheduleType.None -> "None" (que j est no dicionrio)
        
        return localizationService[key];
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        return value; // Geralmente no usado para ComboBox de Enums bidirecional sem lgica extra
    }
}
