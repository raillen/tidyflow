using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using AutoFlow.Domain.Entities;

namespace AutoFlow.Application.Interfaces;

public interface IJobStore
{
    Task<IEnumerable<Job>> GetAllAsync();
    Task<Job?> GetByIdAsync(Guid id);
    Task SaveAsync(Job job);
    Task DeleteAsync(Guid id);
}
