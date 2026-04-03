using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Controls.Notifications;
using FolderFlow.Application.Interfaces;
using FolderFlow.Infrastructure.Helpers;

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
        var appLifetime = Avalonia.Application.Current?.ApplicationLifetime as IClassicDesktopStyleApplicationLifetime;
        var mainWindow = appLifetime?.MainWindow;

        // Se a janela principal existe, est visvel e em foco (ou pelo menos no minimizada)
        if (mainWindow != null && mainWindow.IsVisible && mainWindow.WindowState != WindowState.Minimized)
        {
            EnsureManager();
            if (_notificationManager != null)
            {
                _notificationManager.Show(new Notification(
                    title, 
                    message, 
                    isError ? NotificationType.Error : NotificationType.Information));
                return;
            }
        }

        // Caso contrrio (Minimizado no tray ou Janela nula), envia Notificao Nativa do Windows
        WindowsNotificationHelper.SendToast(title, message);
    }
}
