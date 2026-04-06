using System.IO;
using AutoFlow.Application.Interfaces;

namespace AutoFlow.Infrastructure.Security;

public class EncryptionService : IEncryptionService
{
    public Stream GetEncryptStream(Stream targetStream, string password)
    {
        return EncryptionHelper.GetEncryptStream(targetStream, password);
    }
}
