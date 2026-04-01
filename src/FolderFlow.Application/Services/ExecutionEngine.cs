using System;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;
using System.Threading;
using System.Threading.Tasks;
using System.Diagnostics;
using FolderFlow.Application.Interfaces;
using FolderFlow.Domain.Entities;
using FolderFlow.Domain.Enums;
using FolderFlow.Domain.ValueObjects;

namespace FolderFlow.Application.Services;

public class ExecutionEngine
{
    private readonly IFileOperator _fileOperator;
    private readonly IAppLogger _logger;
    private readonly IHashService _hashService;
    private readonly INotificationService _notificationService;
    private readonly ICloudHydrationService _cloudHydrationService;
    private readonly IAuditService _auditService;
    private readonly IFailureStore _failureStore;
    private readonly ISystemActivityService _activityService;
    private readonly GlobalProgressService _globalProgressService;
    private readonly IExternalNotificationService _externalNotificationService;
    private readonly IScriptRunner _scriptRunner;

    public ExecutionEngine(
        IFileOperator fileOperator, 
        IAppLogger logger, 
        IHashService hashService,
        INotificationService notificationService,
        ICloudHydrationService cloudHydrationService,
        IAuditService auditService,
        IFailureStore failureStore,
        ISystemActivityService activityService,
        GlobalProgressService globalProgressService,
        IExternalNotificationService externalNotificationService,
        IScriptRunner scriptRunner)
    {
        _fileOperator = fileOperator;
        _logger = logger;
        _hashService = hashService;
        _notificationService = notificationService;
        _cloudHydrationService = cloudHydrationService;
        _auditService = auditService;
        _failureStore = failureStore;
        _activityService = activityService;
        _globalProgressService = globalProgressService;
        _externalNotificationService = externalNotificationService;
        _scriptRunner = scriptRunner;
    }

    public async Task RunJobAsync(Job job, CancellationToken cancellationToken = default, bool isRetry = false, IProgress<JobProgressInfo>? progress = null)
    {
        var auditEntries = new System.Collections.Generic.List<AuditEntry>();
        var failedPaths = new System.Collections.Generic.List<string>();
        var progressInfo = new JobProgressInfo { JobId = job.Id, JobName = job.Name };
        var lastReportTime = DateTime.MinValue;
        bool hasCriticalError = false;
        string? errorMessage = null;
        int processedCount = 0;

        void ReportProgress(bool force = false)
        {
            if (force || (DateTime.Now - lastReportTime).TotalMilliseconds > 200)
            {
                progress?.Report(progressInfo);
                _globalProgressService.ReportProgress(progressInfo);
                lastReportTime = DateTime.Now;
            }
        }

        try
        {
            await _activityService.LogActivityAsync($"Tarefa '{job.Name}' iniciada.");
            await _logger.LogAsync($"Iniciando Job: {job.Name}" + (isRetry ? " [MODO RETRY]" : ""));

            if (!string.IsNullOrWhiteSpace(job.PreScriptPath))
            {
                var preSuccess = await _scriptRunner.RunScriptAsync(job.PreScriptPath, cancellationToken);
                if (!preSuccess)
                {
                    await _logger.LogAsync("Pre-script falhou. A tarefa continuar, mas verifique os logs.", "WARNING");
                }
            }

            if (!_fileOperator.Exists(job.SourcePath) && !isRetry)
            {
                var msg = $"Pasta de origem não encontrada: {job.SourcePath}";
                await _logger.LogAsync($"ERRO: {msg}", "ERROR");
                _notificationService.Show("Erro no Job", msg, true);
                auditEntries.Add(new AuditEntry { JobName = job.Name, Status = "FALHA", Details = msg });
                await _auditService.SaveReportAsync(job.Name, auditEntries);
                await _activityService.LogActivityAsync($"Tarefa '{job.Name}' falhou: Origem no encontrada.", "ERROR");
                hasCriticalError = true;
                errorMessage = msg;
                return;
            }

            _fileOperator.CreateDirectory(job.TargetPath);

            System.Collections.Generic.List<string> filesToProcess;
            if (isRetry)
            {
                var failures = await _failureStore.GetFailuresAsync(job.Id);
                filesToProcess = failures.ToList();
            }
            else
            {
                filesToProcess = _fileOperator.EnumerateFiles(job.SourcePath, "*", job.Recursive).ToList();
            }

            progressInfo.TotalFiles = filesToProcess.Count;
            
            // Calcula TotalBytes para ETA
            long totalJobBytes = 0;
            foreach (var f in filesToProcess)
            {
                var meta = _fileOperator.GetFileMetadata(f);
                if (meta != null) totalJobBytes += meta.Size;
            }
            progressInfo.TotalBytes = totalJobBytes;

            foreach (var file in filesToProcess)
            {
                if (cancellationToken.IsCancellationRequested) break;

                progressInfo.CurrentFile = Path.GetFileName(file);
                ReportProgress();

                var fileMeta = _fileOperator.GetFileMetadata(file);
                long fileBytes = fileMeta?.Size ?? 0;

                if (ShouldProcessFile(file, job))
                {
                    // RETRY AUTOMTICO (3 vezes)
                    AuditEntry? entry = null;
                    int attempts = 0;
                    while (attempts < 3)
                    {
                        progressInfo.CurrentFilePercentage = 0;
                        progressInfo.TransferSpeed = 0;
                        
                        long startProcessedBytes = progressInfo.ProcessedBytes; // Salva o estado antes do retry
                        
                        entry = await ProcessFileAsync(file, job, cancellationToken, (p) => 
                        {
                            // Atualiza ProcessedBytes durante a cópia baseando-se no %
                            progressInfo.ProcessedBytes = startProcessedBytes + (long)(fileBytes * (p.CurrentFilePercentage / 100.0));
                            
                            // Calcula ETA simples
                            if (p.TransferSpeed > 0)
                            {
                                var remainingBytes = p.TotalBytes - p.ProcessedBytes;
                                var remainingSeconds = remainingBytes / p.TransferSpeed;
                                p.EstimatedTimeRemaining = TimeSpan.FromSeconds(remainingSeconds);
                            }
                            
                            ReportProgress();
                        }, progressInfo);
                        
                        if (entry != null && entry.Status != "FALHA") 
                        {
                            // Garante que o total do arquivo foi somado ao final do sucesso
                            progressInfo.ProcessedBytes = startProcessedBytes + fileBytes;
                            break;
                        }
                        else
                        {
                            // Restaura ProcessedBytes em caso de falha para tentar de novo
                            progressInfo.ProcessedBytes = startProcessedBytes;
                        }
                        
                        attempts++;
                        if (attempts < 3) await Task.Delay(1000, cancellationToken);
                    }

                    if (entry != null)
                    {
                        auditEntries.Add(entry);
                        if (entry.Status == "FALHA") failedPaths.Add(file);
                        else if (entry.Status != "IGNORADO") processedCount++;

                        progressInfo.Status = entry.Status;
                        progressInfo.Details = entry.Details ?? "";
                    }
                }
                else
                {
                    progressInfo.ProcessedBytes += fileBytes; // Mesmo ignorado, conta como processado no total
                    var details = "Filtro de excluso ou critrios de data/tamanho.";
                    auditEntries.Add(new AuditEntry { JobName = job.Name, SourcePath = file, Status = "IGNORADO", Details = details });
                    progressInfo.Status = "IGNORADO";
                    progressInfo.Details = details;
                }

                progressInfo.ProcessedFiles++;
                progressInfo.Timestamp = DateTime.Now;
                ReportProgress();
            }

            ReportProgress(true); // Final report

            if (failedPaths.Any()) await _failureStore.SaveFailuresAsync(job.Id, failedPaths);
            else await _failureStore.ClearFailuresAsync(job.Id);

            await _auditService.SaveReportAsync(job.Name, auditEntries);
            await _logger.LogAsync($"Job Finalizado: {job.Name}. {processedCount} processados.");
            _notificationService.Show("Job Concluído", $"{job.Name}: {processedCount} processados.");
            await _activityService.LogActivityAsync($"Tarefa '{job.Name}' concluda. {processedCount} arquivos processados.");
            
            if (failedPaths.Any())
            {
                hasCriticalError = true;
                errorMessage = $"{failedPaths.Count} arquivos falharam.";
            }
        }
        catch (OperationCanceledException)
        {
            await _activityService.LogActivityAsync($"Tarefa '{job.Name}' cancelada pelo usurio.", "WARNING");
            hasCriticalError = true;
            errorMessage = "Operação cancelada pelo usuário.";
        }
        catch (Exception ex)
        {
            await _logger.LogAsync($"ERRO CRÍTICO: {ex.Message}", "ERROR");
            _notificationService.Show("Erro Crítico", ex.Message, true);
            await _activityService.LogActivityAsync($"Erro crtico na tarefa '{job.Name}': {ex.Message}", "ERROR");
            hasCriticalError = true;
            errorMessage = ex.Message;
        }
        finally
        {
            if (!string.IsNullOrWhiteSpace(job.PostScriptPath))
            {
                var postSuccess = await _scriptRunner.RunScriptAsync(job.PostScriptPath, cancellationToken);
                if (!postSuccess)
                {
                    await _logger.LogAsync("Post-script falhou.", "WARNING");
                }
            }

            await _externalNotificationService.NotifyJobCompletionAsync(job, !hasCriticalError, processedCount, errorMessage);
            _globalProgressService.CompleteJob(job.Id);
        }
    }

    private bool ShouldProcessFile(string file, Job job)
    {
        var meta = _fileOperator.GetFileMetadata(file);
        if (meta == null) return false;
        
        // Extensões
        if (job.IncludeExtensions.Any())
        {
            var ext = Path.GetExtension(file).ToLower();
            if (!job.IncludeExtensions.Any(e => e.Trim().ToLower() == ext)) return false;
        }

        // Regex
        if (!string.IsNullOrWhiteSpace(job.NameRegex))
        {
            if (!Regex.IsMatch(Path.GetFileName(file), job.NameRegex, RegexOptions.IgnoreCase)) return false;
        }

        // Exclusão
        if (job.ExcludePatterns.Any(p => file.Contains(p, StringComparison.OrdinalIgnoreCase))) return false;

        // Tamanho
        if (job.MinSizeKB.HasValue && meta.Size < job.MinSizeKB * 1024) return false;
        if (job.MaxSizeKB.HasValue && meta.Size > job.MaxSizeKB * 1024) return false;

        // Data
        if (job.ModifiedWithinDays.HasValue)
        {
            if (DateTime.UtcNow - meta.LastWriteTimeUtc > TimeSpan.FromDays(job.ModifiedWithinDays.Value)) return false;
        }

        return true;
    }

    private async Task<AuditEntry?> ProcessFileAsync(string sourceFile, Job job, CancellationToken cancellationToken, Action<JobProgressInfo>? reportAction = null, JobProgressInfo? progressInfo = null)
    {
        var entry = new AuditEntry { JobName = job.Name, SourcePath = sourceFile };
        string? targetFile = null;

        try
        {
            await _cloudHydrationService.EnsureFileIsLocalAsync(sourceFile, cancellationToken);
            var relativePath = Path.GetRelativePath(job.SourcePath, sourceFile);
            targetFile = Path.Combine(job.TargetPath, relativePath);
            entry.TargetPath = targetFile;

            _fileOperator.CreateDirectory(Path.GetDirectoryName(targetFile)!);

            if (_fileOperator.Exists(targetFile))
            {
                if (job.SmartSync)
                {
                    var sm = _fileOperator.GetFileMetadata(sourceFile);
                    var tm = _fileOperator.GetFileMetadata(targetFile);
                    if (sm != null && tm != null && sm.Size == tm.Size && Math.Abs((sm.LastWriteTimeUtc - tm.LastWriteTimeUtc).TotalSeconds) < 2)
                    {
                        entry.Status = "IGNORADO"; entry.Details = "SmartSync"; return entry;
                    }
                }

                switch (job.ConflictMode)
                {
                    case ConflictMode.Skip: entry.Status = "IGNORADO"; return entry;
                    case ConflictMode.Overwrite: await MoveToTrashAsync(targetFile, job, cancellationToken); await _fileOperator.DeleteAsync(targetFile, cancellationToken); break;
                    case ConflictMode.Rename: targetFile = GenerateUniqueFileName(targetFile); entry.TargetPath = targetFile; break;
                }
            }

            if (job.Mode == JobMode.Copy || job.VerifyHash) 
            {
                var meta = _fileOperator.GetFileMetadata(sourceFile);
                var totalBytes = meta?.Size ?? 0;
                var sw = Stopwatch.StartNew();

                var fileProgress = new Progress<double>(p => 
                {
                    if (progressInfo != null)
                    {
                        progressInfo.CurrentFilePercentage = p;
                        var elapsed = sw.Elapsed.TotalSeconds;
                        if (elapsed > 0)
                        {
                            progressInfo.TransferSpeed = (totalBytes * (p / 100)) / elapsed;
                        }
                        reportAction?.Invoke(progressInfo);
                    }
                });
                await _fileOperator.CopyAsync(sourceFile, targetFile, cancellationToken, fileProgress);
                sw.Stop();
            }
            else 
            {
                await _fileOperator.MoveAsync(sourceFile, targetFile, cancellationToken);
            }

            // Verificao de Integridade
            if (job.VerifyHash)
            {
                var sourceHash = await _hashService.ComputeHashAsync(sourceFile);
                var targetHash = await _hashService.ComputeHashAsync(targetFile);

                if (sourceHash != targetHash)
                {
                    await _fileOperator.DeleteAsync(targetFile, CancellationToken.None); // Sem token para garantir deleo
                    entry.Status = "FALHA";
                    entry.Details = "Falha na verificao de integridade (Hash mismatch).";
                    return entry;
                }

                // Se era MOVE, deleta a origem agora que o hash bateu
                if (job.Mode == JobMode.Move)
                {
                    await _fileOperator.DeleteAsync(sourceFile, CancellationToken.None);
                }
            }

            entry.Status = (job.Mode == JobMode.Copy) ? "COPIADO" : "MOVIDO";
            return entry;
        }
        catch (OperationCanceledException)
        {
            // LIMPEZA SEGURA: Se cancelado, remove arquivo de destino parcial para evitar corrupo
            if (targetFile != null && _fileOperator.Exists(targetFile))
            {
                await _fileOperator.DeleteAsync(targetFile, CancellationToken.None);
            }
            throw; // Repropagar para o RunJobAsync tratar
        }
        catch (Exception ex) 
        { 
            // Limpeza em caso de erro genrico se o arquivo destino existe
            if (targetFile != null && _fileOperator.Exists(targetFile))
            {
                await _fileOperator.DeleteAsync(targetFile, CancellationToken.None);
            }
            entry.Status = "FALHA"; 
            entry.Details = ex.Message; 
            return entry; 
        }
    }

    private async Task MoveToTrashAsync(string filePath, Job job, CancellationToken cancellationToken)
    {
        if (!job.EnableTrash) return;
        try
        {
            var trashFolder = Path.Combine(job.TargetPath, ".folderflow", "trash", DateTime.Now.ToString("yyyyMMdd"));
            _fileOperator.CreateDirectory(trashFolder);
            await _fileOperator.CopyAsync(filePath, Path.Combine(trashFolder, $"{Guid.NewGuid()}_{Path.GetFileName(filePath)}"), cancellationToken);
        }
        catch { }
    }

    private string GenerateUniqueFileName(string filePath)
    {
        var dir = Path.GetDirectoryName(filePath);
        var fileName = Path.GetFileNameWithoutExtension(filePath);
        var ext = Path.GetExtension(filePath);
        var count = 1;
        string newPath;
        do { newPath = Path.Combine(dir ?? string.Empty, $"{fileName}_{count}{ext}"); count++; } while (_fileOperator.Exists(newPath));
        return newPath;
    }
}
