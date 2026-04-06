using System.Collections.Generic;
using System.Threading.Tasks;
using AutoFlow.Domain.Entities;

namespace AutoFlow.Application.Interfaces;

public interface IAuditService
{
    Task SaveReportAsync(string jobName, IEnumerable<AuditEntry> entries);
    Task<int> PurgeOldLogsAsync(int days);
    Task<string> GetDailySummaryAsync();
}
