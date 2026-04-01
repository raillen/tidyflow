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

    private static PerformanceCounter? _cpuCounter;

    public static double GetCpuUsage()
    {
        try
        {
            if (_cpuCounter == null)
            {
                _cpuCounter = new PerformanceCounter("Processor", "% Processor Time", "_Total");
                _cpuCounter.NextValue(); // Primeira leitura  sempre 0
            }
            return _cpuCounter.NextValue();
        }
        catch { return 0; }
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
