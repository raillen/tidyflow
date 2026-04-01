using System.IO;

namespace FolderFlow.Application.Interfaces;

public interface IEncryptionService
{
    Stream GetEncryptStream(Stream targetStream, string password);
}
