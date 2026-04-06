using Avalonia;
using Avalonia.Styling;
using AutoFlow.Domain.Enums;
using Avalonia.Media;

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

    public void SetFont(AppFont font)
    {
        if (Avalonia.Application.Current == null) return;

        var fontFamilyName = font switch
        {
            AppFont.System => "font-family: Inter, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif",
            AppFont.Roboto => "Roboto, sans-serif",
            AppFont.Montserrat => "Montserrat, sans-serif",
            AppFont.OpenSans => "Open Sans, sans-serif",
            AppFont.PublicSans => "Public Sans, sans-serif",
            AppFont.JetBrains => "JetBrains Mono, sans-serif",
            _ => "Inter, sans-serif" // Default
        };

        // Atualiza a fonte global nos recursos da aplicação
        // O Semi.Avalonia usa a FontFamily padrão, então mudar na raiz deve propagar
        Avalonia.Application.Current.Resources["ContentControlThemeFontFamily"] = new FontFamily(fontFamilyName);
        Avalonia.Application.Current.Resources["DefaultFontFamily"] = new FontFamily(fontFamilyName);
    }
}
