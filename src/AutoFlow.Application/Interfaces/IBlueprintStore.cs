using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using AutoFlow.Domain.Entities;

namespace AutoFlow.Application.Interfaces;

public interface IBlueprintStore
{
    Task<IEnumerable<Blueprint>> GetAllAsync();
    Task<Blueprint?> GetByIdAsync(Guid id);
    Task SaveAsync(Blueprint blueprint);
    Task DeleteAsync(Guid id);
}
