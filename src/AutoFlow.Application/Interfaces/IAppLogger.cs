using System.Threading.Tasks;

namespace AutoFlow.Application.Interfaces;

public interface IAppLogger
{
    Task LogAsync(string message, string level = "INFO");
}
