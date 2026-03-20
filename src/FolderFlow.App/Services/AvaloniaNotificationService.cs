using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Controls.Notifications;
using FolderFlow.Application.Interfaces;

namespace FolderFlow.App.Services;

public class AvaloniaNotificationService : INotificationService
{
    private WindowNotificationManager? _notificationManager;

    private void EnsureManager()
    {
        if (_notificationManager != null) return;

        if (Avalonia.Application.Current?.ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop && desktop.MainWindow != null)
        {
            _notificationManager = new WindowNotificationManager(desktop.MainWindow)
            {
                Position = NotificationPosition.BottomRight,
                MaxItems = 3
            };
        }
    }

    public void Show(string title, string message, bool isError = false)
    {
        EnsureManager();
        if (_notificationManager == null) return;

        _notificationManager.Show(new Notification(
            title, 
            message, 
            isError ? NotificationType.Error : NotificationType.Information));
    }
}
