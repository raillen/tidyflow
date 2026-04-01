using System.IO;
using FolderFlow.Application.Interfaces;

namespace FolderFlow.Infrastructure.Security;

public class EncryptionService : IEncryptionService
{
    public Stream GetEncryptStream(Stream targetStream, string password)
    {
        return EncryptionHelper.GetEncryptStream(targetStream, password);
    }
}
