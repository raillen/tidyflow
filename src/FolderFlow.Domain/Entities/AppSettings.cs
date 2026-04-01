using FolderFlow.Domain.Enums;

namespace FolderFlow.Domain.Entities;

public class AppSettings
{
    public ThemeMode Theme { get; set; } = ThemeMode.System;
    public string Language { get; set; } = "pt-BR";
    public bool ShowNotifications { get; set; } = true;
    
    // Performance
    public int MaxDegreeOfParallelism { get; set; } = 2;
    public string ProcessPriority { get; set; } = "Normal"; // Normal, BelowNormal, High
    public long BandwidthLimitBytes { get; set; } = 0; // 0 = Sem limite

    // Windows Integration
    public bool StartAtStartup { get; set; } = false;
    public bool StartMinimized { get; set; } = false;
    public bool CloseToTray { get; set; } = true;
    
    // Maintenance
    public int LogRetentionDays { get; set; } = 30; // 0 = Para sempre
    
    // Notifications Pro
    public string WebhookUrl { get; set; } = string.Empty;
    public string WebhookType { get; set; } = "Generic"; // Generic, Discord, Slack
    public bool NotifyOnSuccess { get; set; } = false;
    public bool NotifyOnError { get; set; } = true;

    // SMTP (Email)
    public string SmtpHost { get; set; } = string.Empty;
    public int SmtpPort { get; set; } = 587;
    public string SmtpUser { get; set; } = string.Empty;
    public string SmtpPass { get; set; } = string.Empty;
    public string NotificationEmail { get; set; } = string.Empty;
    public bool EnableSmtp { get; set; } = false;

    // Security
    public string AppPin { get; set; } = string.Empty; // PIN de 4-6 dgitos
    public string MasterEncryptionKey { get; set; } = string.Empty;
    public bool LockOnMinimize { get; set; } = false;

    public double GlassOpacity { get; set; } = 0.6;
    public string GlassMaterial { get; set; } = "Mica";
    public int SystemMonitorUpdateIntervalSeconds { get; set; } = 2;
}
