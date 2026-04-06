using System;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using AutoFlow.Application.Interfaces;

namespace AutoFlow.Application.Services;

public class RollbackEngine
{
    private readonly IRollbackStore _rollbackStore;
    private readonly FileOperatorFactory _fileOperatorFactory;
    private readonly IAppLogger _logger;
    private readonly ISystemActivityService _activityService;

    public RollbackEngine(
        IRollbackStore rollbackStore,
        FileOperatorFactory fileOperatorFactory,
        IAppLogger logger,
        ISystemActivityService activityService)
    {
        _rollbackStore = rollbackStore;
        _fileOperatorFactory = fileOperatorFactory;
        _logger = logger;
        _activityService = activityService;
    }

    public async Task<bool> RollbackAsync(Guid jobId, CancellationToken cancellationToken = default)
    {
        var manifest = await _rollbackStore.GetLatestManifestAsync(jobId);
        if (manifest == null || manifest.Items.Count == 0) return false;

        await _activityService.LogActivityAsync($"Iniciando Rollback para o fluxo: {manifest.JobName}");
        
        int successCount = 0;
        foreach (var item in manifest.Items)
        {
            if (cancellationToken.IsCancellationRequested) break;

            var fileOp = _fileOperatorFactory.GetOperator(item.TargetPath, item.SourcePath);
            
            try
            {
                if (item.Action == "MOVIDO")
                {
                    if (fileOp.Exists(item.TargetPath))
                    {
                        var dir = Path.GetDirectoryName(item.SourcePath);
                        if (!string.IsNullOrWhiteSpace(dir)) fileOp.CreateDirectory(dir);
                        
                        await fileOp.MoveAsync(item.TargetPath, item.SourcePath, cancellationToken);
                        successCount++;
                    }
                }
                else if (item.Action == "COPIADO")
                {
                    if (fileOp.Exists(item.TargetPath))
                    {
                        await fileOp.DeleteAsync(item.TargetPath, cancellationToken);
                        successCount++;
                    }
                }
            }
            catch (Exception ex)
            {
                await _logger.LogAsync($"Erro ao reverter {item.TargetPath}: {ex.Message}", "ERROR");
            }
        }

        await _rollbackStore.ClearManifestAsync(jobId);
        await _activityService.LogActivityAsync($"Rollback concluído: {manifest.JobName} ({successCount} itens revertidos)");
        return true;
    }
}
