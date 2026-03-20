using System.IO;
using System.Threading.Tasks;

namespace FolderFlow.Application.Interfaces;

public interface IHashService
{
    Task<string> ComputeHashAsync(string filePath);
}
