using Microsoft.Win32;
using System;
using System.IO;

namespace FolderFlow.Infrastructure.Helpers;

public static class WindowsStartupHelper
{
    private const string AppName = "FolderFlow";
    private const string RunKey = @"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";

    public static void SetStartup(bool enable)
    {
        try
        {
            using var key = Registry.CurrentUser.OpenSubKey(RunKey, true);
            if (key == null) return;

            if (enable)
            {
                var exePath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "FolderFlow.App.exe");
                if (File.Exists(exePath))
                {
                    key.SetValue(AppName, $"\"{exePath}\" --minimized");
                }
            }
            else
            {
                key.DeleteValue(AppName, false);
            }
        }
        catch { }
    }

    public static void SetProcessPriority(string priority)
    {
        try
        {
            var process = System.Diagnostics.Process.GetCurrentProcess();
            process.PriorityClass = priority switch
            {
                "High" => System.Diagnostics.ProcessPriorityClass.High,
                "BelowNormal" => System.Diagnostics.ProcessPriorityClass.BelowNormal,
                _ => System.Diagnostics.ProcessPriorityClass.Normal
            };
        }
        catch { }
    }
}
