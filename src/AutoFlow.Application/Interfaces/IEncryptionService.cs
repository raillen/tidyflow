using System.IO;

namespace AutoFlow.Application.Interfaces;

public interface IEncryptionService
{
    Stream GetEncryptStream(Stream targetStream, string password);
}
