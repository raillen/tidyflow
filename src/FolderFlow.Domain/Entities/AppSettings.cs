using FolderFlow.Domain.Enums;

namespace FolderFlow.Domain.Entities;

public class AppSettings
{
    public ThemeMode Theme { get; set; } = ThemeMode.System;
    public string Language { get; set; } = "pt-BR";
    public bool ShowNotifications { get; set; } = true;
    public bool StartAtStartup { get; set; } = false;
    public double GlassOpacity { get; set; } = 0.6;
    public string GlassMaterial { get; set; } = "Mica";
    public bool CloseToTray { get; set; } = true;
    public int SystemMonitorUpdateIntervalSeconds { get; set; } = 2;
}
