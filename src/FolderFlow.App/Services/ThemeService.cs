using Avalonia;
using Avalonia.Styling;
using FolderFlow.Domain.Enums;

namespace FolderFlow.App.Services;

public class ThemeService
{
    public void SetTheme(ThemeMode theme)
    {
        if (Avalonia.Application.Current == null) return;

        Avalonia.Application.Current.RequestedThemeVariant = theme switch
        {
            ThemeMode.Light => ThemeVariant.Light,
            ThemeMode.Dark => ThemeVariant.Dark,
            _ => ThemeVariant.Default
        };
    }
}
