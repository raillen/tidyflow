using System;
using System.Collections.Generic;
using System.Linq;

namespace AutoFlow.Application.Helpers;

public class SpeedTracker
{
    private readonly Queue<(DateTime time, long bytes)> _samples = new();
    private readonly int _windowSeconds;

    public SpeedTracker(int windowSeconds = 3)
    {
        _windowSeconds = windowSeconds;
    }

    public void AddSample(long totalProcessedBytes)
    {
        var now = DateTime.Now;
        _samples.Enqueue((now, totalProcessedBytes));

        // Remove amostras fora da janela de tempo
        var cutoff = now.AddSeconds(-_windowSeconds);
        while (_samples.Count > 2 && _samples.Peek().time < cutoff)
        {
            _samples.Dequeue();
        }
    }

    public double GetAverageSpeed()
    {
        if (_samples.Count < 2) return 0;

        var oldest = _samples.Peek();
        var newest = _samples.Last();

        var timeDiff = (newest.time - oldest.time).TotalSeconds;
        if (timeDiff <= 0) return 0;

        var bytesDiff = newest.bytes - oldest.bytes;
        return bytesDiff / timeDiff;
    }

    public void Reset()
    {
        _samples.Clear();
    }
}
