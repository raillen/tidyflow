using FolderFlow.Domain.Enums;

namespace FolderFlow.Domain.Entities;

public class AppSettings
{
    public ThemeMode Theme { get; set; } = ThemeMode.System;
    public string Language { get; set; } = "pt-BR";
    public bool ShowNotifications { get; set; } = true;
    public bool StartAtStartup { get; set; } = false;
}
