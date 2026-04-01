using System;
using System.IO;
using System.Security.Cryptography;
using System.Text;

namespace FolderFlow.Infrastructure.Security;

public static class EncryptionHelper
{
    private static readonly byte[] Salt = Encoding.UTF8.GetBytes("FolderFlowSecureSalt2026!");

    public static CryptoStream GetEncryptStream(Stream targetStream, string password)
    {
        var keyAndIv = Rfc2898DeriveBytes.Pbkdf2(
            Encoding.UTF8.GetBytes(password),
            Salt,
            100000,
            HashAlgorithmName.SHA256,
            48); // 32 bytes for Key, 16 bytes for IV

        using var aes = Aes.Create();
        
        var key = new byte[32];
        var iv = new byte[16];
        Buffer.BlockCopy(keyAndIv, 0, key, 0, 32);
        Buffer.BlockCopy(keyAndIv, 32, iv, 0, 16);

        aes.Key = key;
        aes.IV = iv;
        
        return new CryptoStream(targetStream, aes.CreateEncryptor(), CryptoStreamMode.Write);
    }
}
