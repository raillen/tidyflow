using System.Threading;
using System.Threading.Tasks;

namespace AutoFlow.Application.Interfaces;

public interface IScriptRunner
{
    Task<bool> RunScriptAsync(string scriptPath, CancellationToken cancellationToken = default);
}
