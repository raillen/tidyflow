using System.Collections.Generic;
using System.Threading.Tasks;
using FolderFlow.Domain.Entities;

namespace FolderFlow.Application.Interfaces;

public interface IAuditService
{
    Task SaveReportAsync(string jobName, IEnumerable<AuditEntry> entries);
}
