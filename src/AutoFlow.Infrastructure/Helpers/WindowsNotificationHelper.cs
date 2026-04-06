using System;
using System.Diagnostics;
using System.IO;

namespace AutoFlow.Infrastructure.Helpers;

public static class WindowsNotificationHelper
{
    public static void SendToast(string title, string message)
    {
        try
        {
            // Escapa aspas para o PowerShell
            string safeTitle = title.Replace("\"", "'");
            string safeMessage = message.Replace("\"", "'");

            // Script PowerShell que usa WinRT para mostrar notificação nativa
            // Funciona no Windows 10 e 11 sem dependências externas
            string script = $@"
$ErrorActionPreference = 'SilentlyContinue'
$appId = 'AutoFlow'
[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
[Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null

$template = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02)
$textNodes = $template.GetElementsByTagName('text')
$textNodes.Item(0).AppendChild($template.CreateTextNode('{safeTitle}')) | Out-Null
$textNodes.Item(1).AppendChild($template.CreateTextNode('{safeMessage}')) | Out-Null

$toast = [Windows.UI.Notifications.ToastNotification]::new($template)
[Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier($appId).Show($toast)
";

            var startInfo = new ProcessStartInfo
            {
                FileName = "powershell.exe",
                Arguments = $"-NoProfile -ExecutionPolicy Bypass -Command \"{script.Replace("\r\n", " ")}\"",
                UseShellExecute = false,
                CreateNoWindow = true,
                WindowStyle = ProcessWindowStyle.Hidden
            };

            Process.Start(startInfo);
        }
        catch (Exception ex)
        {
            Debug.WriteLine($"Falha ao enviar notificação nativa: {ex.Message}");
        }
    }
}
