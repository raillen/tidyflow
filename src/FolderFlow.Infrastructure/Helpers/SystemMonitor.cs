using System;
using System.Diagnostics;
using System.Runtime.InteropServices;

namespace FolderFlow.Infrastructure.Helpers;

public static class SystemMonitor
{
    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Auto)]
    private class MEMORYSTATUSEX
    {
        public uint dwLength;
        public uint dwMemoryLoad;
        public ulong ullTotalPhys;
        public ulong ullAvailPhys;
        public ulong ullTotalPageFile;
        public ulong ullAvailPageFile;
        public ulong ullTotalVirtual;
        public ulong ullAvailVirtual;
        public ulong ullAvailExtendedVirtual;
        public MEMORYSTATUSEX() { dwLength = (uint)Marshal.SizeOf(typeof(MEMORYSTATUSEX)); }
    }

    [return: MarshalAs(UnmanagedType.Bool)]
    [DllImport("kernel32.dll", CharSet = CharSet.Auto, SetLastError = true)]
    private static extern bool GlobalMemoryStatusEx([In, Out] MEMORYSTATUSEX lpBuffer);

    private static DateTime _lastTime;
    private static TimeSpan _lastProcessorTime;
    private static double _lastCpuUsage;

    public static double GetCpuUsage()
    {
        try
        {
            var process = Process.GetCurrentProcess();
            var currentTime = DateTime.UtcNow;
            var currentProcessorTime = process.TotalProcessorTime;

            if (_lastTime == DateTime.MinValue)
            {
                _lastTime = currentTime;
                _lastProcessorTime = currentProcessorTime;
                return 0;
            }

            double cpuUsedMs = (currentProcessorTime - _lastProcessorTime).TotalMilliseconds;
            double totalMsPassed = (currentTime - _lastTime).TotalMilliseconds;
            double cpuUsage = (cpuUsedMs / (Environment.ProcessorCount * totalMsPassed)) * 100;

            _lastTime = currentTime;
            _lastProcessorTime = currentProcessorTime;
            _lastCpuUsage = Math.Clamp(cpuUsage, 0, 100);
            
            return _lastCpuUsage;
        }
        catch { return _lastCpuUsage; }
    }

    public static (long total, long used, double percentage) GetRamStatus()
    {
        var memStatus = new MEMORYSTATUSEX();
        if (GlobalMemoryStatusEx(memStatus))
        {
            long total = (long)memStatus.ullTotalPhys;
            long used = total - (long)memStatus.ullAvailPhys;
            return (total, used, memStatus.dwMemoryLoad);
        }
        return (0, 0, 0);
    }
}
