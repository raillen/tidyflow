using Avalonia;
using Avalonia.Styling;
using AutoFlow.Domain.Enums;

namespace AutoFlow.App.Services;

public class CustomThemeVariants
{
    public static readonly ThemeVariant Dracula = new ThemeVariant("Dracula", ThemeVariant.Dark);
    public static readonly ThemeVariant Neon = new ThemeVariant("Neon", ThemeVariant.Dark);
}

public class ThemeService
{
    public void SetTheme(ThemeMode theme)
    {
        if (Avalonia.Application.Current == null) return;

        Avalonia.Application.Current.RequestedThemeVariant = theme switch
        {
            ThemeMode.Light => ThemeVariant.Light,
            ThemeMode.Dark => ThemeVariant.Dark,
            ThemeMode.Dracula => CustomThemeVariants.Dracula,
            ThemeMode.Neon => CustomThemeVariants.Neon,
            _ => ThemeVariant.Default
        };
    }
}
