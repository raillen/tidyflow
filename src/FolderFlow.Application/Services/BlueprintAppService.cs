using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using FolderFlow.Application.Interfaces;
using FolderFlow.Domain.Entities;

namespace FolderFlow.Application.Services;

public class BlueprintAppService
{
    private readonly IBlueprintStore _store;
    private readonly IOrganizationService _organizationService;
    private readonly IAppLogger _logger;

    public BlueprintAppService(IBlueprintStore store, IOrganizationService organizationService, IAppLogger logger)
    {
        _store = store;
        _organizationService = organizationService;
        _logger = logger;
    }

    public Task<IEnumerable<Blueprint>> GetAllBlueprintsAsync() => _store.GetAllAsync();

    public async Task SaveBlueprintAsync(Blueprint blueprint)
    {
        await _store.SaveAsync(blueprint);
        await _logger.LogAsync($"Blueprint '{blueprint.Name}' salvo com sucesso.", "INFO");
    }

    public async Task DeleteBlueprintAsync(Guid id)
    {
        await _store.DeleteAsync(id);
        await _logger.LogAsync($"Blueprint removido: {id}", "INFO");
    }

    public async Task ApplyBlueprintAsync(Blueprint blueprint, string? eventPath = null)
    {
        await _organizationService.ProcessBlueprintAsync(blueprint, eventPath);
    }
}
