using System.Threading;
using System.Threading.Tasks;

namespace FolderFlow.Application.Interfaces;

public interface ISchedulerService
{
    void Start(CancellationToken cancellationToken);
}
