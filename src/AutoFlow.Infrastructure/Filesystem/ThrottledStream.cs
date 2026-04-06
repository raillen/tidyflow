using System;
using System.IO;
using System.Threading;
using System.Threading.Tasks;

namespace AutoFlow.Infrastructure.Filesystem;

public class ThrottledStream : Stream
{
    private readonly Stream _baseStream;
    private readonly long _bytesPerSecond;
    private long _totalBytesRead;
    private readonly DateTime _startTime;

    public ThrottledStream(Stream baseStream, long bytesPerSecond)
    {
        _baseStream = baseStream;
        _bytesPerSecond = bytesPerSecond;
        _startTime = DateTime.Now;
    }

    public override async ValueTask<int> ReadAsync(Memory<byte> buffer, CancellationToken cancellationToken = default)
    {
        int read = await _baseStream.ReadAsync(buffer, cancellationToken);
        if (read > 0 && _bytesPerSecond > 0)
        {
            _totalBytesRead += read;
            double elapsed = (DateTime.Now - _startTime).TotalSeconds;
            if (elapsed > 0)
            {
                double currentBps = _totalBytesRead / elapsed;
                if (currentBps > _bytesPerSecond)
                {
                    double expectedDuration = (double)_totalBytesRead / _bytesPerSecond;
                    double sleepTime = expectedDuration - elapsed;
                    if (sleepTime > 0.01)
                    {
                        await Task.Delay((int)(sleepTime * 1000), cancellationToken);
                    }
                }
            }
        }
        return read;
    }

    // Proxy simples para outros membros
    public override bool CanRead => _baseStream.CanRead;
    public override bool CanSeek => _baseStream.CanSeek;
    public override bool CanWrite => _baseStream.CanWrite;
    public override long Length => _baseStream.Length;
    public override long Position { get => _baseStream.Position; set => _baseStream.Position = value; }
    public override void Flush() => _baseStream.Flush();
    public override int Read(byte[] buffer, int offset, int count) => _baseStream.Read(buffer, offset, count);
    public override long Seek(long offset, SeekOrigin origin) => _baseStream.Seek(offset, origin);
    public override void SetLength(long value) => _baseStream.SetLength(value);
    public override void Write(byte[] buffer, int offset, int count) => _baseStream.Write(buffer, offset, count);
}
