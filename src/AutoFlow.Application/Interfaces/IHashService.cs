using System.IO;
using System.Threading.Tasks;

namespace AutoFlow.Application.Interfaces;

public interface IHashService
{
    Task<string> ComputeHashAsync(string filePath);
}
