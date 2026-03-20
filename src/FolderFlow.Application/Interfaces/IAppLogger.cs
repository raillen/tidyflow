using System.Threading.Tasks;

namespace FolderFlow.Application.Interfaces;

public interface IAppLogger
{
    Task LogAsync(string message, string level = "INFO");
}
