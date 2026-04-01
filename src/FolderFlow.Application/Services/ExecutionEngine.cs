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
    private readonly FileOperatorFactory _fileOperatorFactory;
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
    private readonly IEncryptionService _encryptionService;
    private readonly ISettingsStore _settingsStore;

    public ExecutionEngine(
        FileOperatorFactory fileOperatorFactory, 
        IAppLogger logger, 
        IHashService hashService,
        INotificationService notificationService,
        ICloudHydrationService cloudHydrationService,
        IAuditService auditService,
        IFailureStore failureStore,
        ISystemActivityService activityService,
        GlobalProgressService globalProgressService,
        IExternalNotificationService externalNotificationService,
        IScriptRunner scriptRunner,
        IEncryptionService encryptionService,
        ISettingsStore settingsStore)
    {
        _fileOperatorFactory = fileOperatorFactory;
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
        _encryptionService = encryptionService;
        _settingsStore = settingsStore;
    }

    public async Task RunJobAsync(Job job, CancellationToken cancellationToken = default, bool isRetry = false, IProgress<JobProgressInfo>? progress = null)
    {
        var _fileOperator = _fileOperatorFactory.GetOperator(job.SourcePath, job.TargetPath);
        
        // Aplica limite de banda das configuraes
        var settings = await _settingsStore.LoadAsync();
        _fileOperator.BandwidthLimit = settings.BandwidthLimitBytes;
        
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

            if (job.ArchiveFormat == ArchiveFormat.Zip)
            {
                await RunZipBackupAsync(job, filesToProcess, _fileOperator, cancellationToken, progressInfo, ReportProgress, auditEntries, failedPaths);
                processedCount = filesToProcess.Count - failedPaths.Count;
            }
            else
            {
                foreach (var file in filesToProcess)
                {
                    if (cancellationToken.IsCancellationRequested) break;

                    progressInfo.CurrentFile = Path.GetFileName(file);
                    ReportProgress();

                    var fileMeta = _fileOperator.GetFileMetadata(file);
                    long fileBytes = fileMeta?.Size ?? 0;

                    if (ShouldProcessFile(file, job, _fileOperator))
                    {
                        // RETRY AUTOMTICO (3 vezes)
                        AuditEntry? entry = null;
                        int attempts = 0;
                        while (attempts < 3)
                        {
                            progressInfo.CurrentFilePercentage = 0;
                            progressInfo.TransferSpeed = 0;
                            
                            long startProcessedBytes = progressInfo.ProcessedBytes; // Salva o estado antes do retry
                            
                            entry = await ProcessFileAsync(file, job, _fileOperator, cancellationToken, (p) => 
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
            }

            ReportProgress(true); // Final report

            if (failedPaths.Any()) await _failureStore.SaveFailuresAsync(job.Id, failedPaths);
            else await _failureStore.ClearFailuresAsync(job.Id);

            await _auditService.SaveReportAsync(job.Name, auditEntries);
            await _logger.LogAsync($"Job Finalizado: {job.Name}. {processedCount} processados.");
            _notificationService.Show("Job Concluído", $"{job.Name}: {processedCount} processados.");
            await _activityService.LogActivityAsync($"Tarefa '{job.Name}' concluda. {processedCount} arquivos processados.");
            
            // Applica Retenção
            if (job.RetentionPolicy != RetentionPolicy.None)
            {
                await ApplyRetentionPolicyAsync(job, _fileOperator);
            }

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

    private bool ShouldProcessFile(string file, Job job, IFileOperator fileOperator)
    {
        var meta = fileOperator.GetFileMetadata(file);
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

    private async Task<AuditEntry?> ProcessFileAsync(string sourceFile, Job job, IFileOperator fileOperator, CancellationToken cancellationToken, Action<JobProgressInfo>? reportAction = null, JobProgressInfo? progressInfo = null)
    {
        var entry = new AuditEntry { JobName = job.Name, SourcePath = sourceFile };
        string? targetFile = null;

        try
        {
            await _cloudHydrationService.EnsureFileIsLocalAsync(sourceFile, cancellationToken);
            var relativePath = Path.GetRelativePath(job.SourcePath, sourceFile);
            targetFile = Path.Combine(job.TargetPath, relativePath);
            entry.TargetPath = targetFile;

            fileOperator.CreateDirectory(Path.GetDirectoryName(targetFile)!);

            if (fileOperator.Exists(targetFile))
            {
                if (job.SmartSync)
                {
                    var sm = fileOperator.GetFileMetadata(sourceFile);
                    var tm = fileOperator.GetFileMetadata(targetFile);
                    if (sm != null && tm != null && sm.Size == tm.Size && Math.Abs((sm.LastWriteTimeUtc - tm.LastWriteTimeUtc).TotalSeconds) < 2)
                    {
                        entry.Status = "IGNORADO"; entry.Details = "SmartSync"; return entry;
                    }
                }

                switch (job.ConflictMode)
                {
                    case ConflictMode.Skip: entry.Status = "IGNORADO"; return entry;
                    case ConflictMode.Overwrite: await MoveToTrashAsync(targetFile, job, fileOperator, cancellationToken); await fileOperator.DeleteAsync(targetFile, cancellationToken); break;
                    case ConflictMode.Rename: targetFile = GenerateUniqueFileName(targetFile, fileOperator); entry.TargetPath = targetFile; break;
                }
            }

            var sw = Stopwatch.StartNew();
            if (job.Mode == JobMode.Copy || job.VerifyHash) 
            {
                var meta = fileOperator.GetFileMetadata(sourceFile);
                var totalBytes = meta?.Size ?? 0;
                entry.FileSize = totalBytes;

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
                await fileOperator.CopyAsync(sourceFile, targetFile, cancellationToken, fileProgress, job.EncryptionKey, job.DeltaSync);
            }
            else 
            {
                var meta = fileOperator.GetFileMetadata(sourceFile);
                entry.FileSize = meta?.Size ?? 0;
                await fileOperator.MoveAsync(sourceFile, targetFile, cancellationToken);
            }
            sw.Stop();
            entry.DurationMs = sw.Elapsed.TotalMilliseconds;

            // Verificao de Integridade
            if (job.VerifyHash)
            {
                var sourceHash = await _hashService.ComputeHashAsync(sourceFile);
                var targetHash = await _hashService.ComputeHashAsync(targetFile);

                if (sourceHash != targetHash)
                {
                    await fileOperator.DeleteAsync(targetFile, CancellationToken.None); // Sem token para garantir deleo
                    entry.Status = "FALHA";
                    entry.Details = "Falha na verificao de integridade (Hash mismatch).";
                    return entry;
                }

                // Se era MOVE, deleta a origem agora que o hash bateu
                if (job.Mode == JobMode.Move)
                {
                    await fileOperator.DeleteAsync(sourceFile, CancellationToken.None);
                }
            }

            entry.Status = (job.Mode == JobMode.Copy) ? "COPIADO" : "MOVIDO";
            return entry;
        }
        catch (OperationCanceledException)
        {
            // LIMPEZA SEGURA: Se cancelado, remove arquivo de destino parcial para evitar corrupo
            if (targetFile != null && fileOperator.Exists(targetFile))
            {
                await fileOperator.DeleteAsync(targetFile, CancellationToken.None);
            }
            throw; // Repropagar para o RunJobAsync tratar
        }
        catch (Exception ex) 
        { 
            // Limpeza em caso de erro genrico se o arquivo destino existe
            if (targetFile != null && fileOperator.Exists(targetFile))
            {
                await fileOperator.DeleteAsync(targetFile, CancellationToken.None);
            }
            entry.Status = "FALHA"; 
            entry.Details = ex.Message; 
            return entry; 
        }
    }

    private async Task RunZipBackupAsync(Job job, System.Collections.Generic.List<string> filesToProcess, IFileOperator fileOperator, CancellationToken cancellationToken, JobProgressInfo progressInfo, Action<bool> reportProgress, System.Collections.Generic.List<AuditEntry> auditEntries, System.Collections.Generic.List<string> failedPaths)
    {
        var timestamp = DateTime.Now.ToString("yyyyMMdd_HHmmss");
        var zipFileName = $"{job.Name.Replace(" ", "_")}_{timestamp}.zip";
        var targetZipPath = Path.Combine(job.TargetPath, zipFileName);

        try
        {
            await using var zipFileStream = new FileStream(targetZipPath, FileMode.Create, FileAccess.Write, FileShare.None, 4096, true);
            Stream baseStream = zipFileStream;
            
            // Envolve em CryptoStream se tiver chave (ateno: isso encripta o arquivo zip inteiro, no os arquivos dentro do zip padrão)
            // Uma abordagem de criptografar o contêiner final para backups.
            if (!string.IsNullOrWhiteSpace(job.EncryptionKey))
            {
                baseStream = _encryptionService.GetEncryptStream(zipFileStream, job.EncryptionKey);
            }

            using var archive = new System.IO.Compression.ZipArchive(baseStream, System.IO.Compression.ZipArchiveMode.Create);
            
            var sw = Stopwatch.StartNew();

            foreach (var file in filesToProcess)
            {
                if (cancellationToken.IsCancellationRequested) break;

                progressInfo.CurrentFile = Path.GetFileName(file);
                reportProgress(false);

                if (ShouldProcessFile(file, job, fileOperator))
                {
                    try
                    {
                        var entryName = Path.GetRelativePath(job.SourcePath, file);
                        var entry = archive.CreateEntry(entryName, System.IO.Compression.CompressionLevel.Optimal);
                        
                        await using var entryStream = entry.Open();
                        await using var sourceStream = new FileStream(file, FileMode.Open, FileAccess.Read, FileShare.Read, 4096, true);

                        var totalBytes = sourceStream.Length;
                        var buffer = new byte[1024 * 1024];
                        long totalRead = 0;
                        int bytesRead;

                        while ((bytesRead = await sourceStream.ReadAsync(buffer, 0, buffer.Length, cancellationToken)) > 0)
                        {
                            await entryStream.WriteAsync(buffer, 0, bytesRead, cancellationToken);
                            totalRead += bytesRead;
                            progressInfo.ProcessedBytes += bytesRead;

                            var elapsed = sw.Elapsed.TotalSeconds;
                            if (elapsed > 0)
                            {
                                progressInfo.TransferSpeed = progressInfo.ProcessedBytes / elapsed;
                                var remainingBytes = progressInfo.TotalBytes - progressInfo.ProcessedBytes;
                                progressInfo.EstimatedTimeRemaining = TimeSpan.FromSeconds(remainingBytes / progressInfo.TransferSpeed);
                            }

                            progressInfo.CurrentFilePercentage = totalBytes > 0 ? (double)totalRead / totalBytes * 100 : 100;
                            reportProgress(false);
                        }

                        auditEntries.Add(new AuditEntry { JobName = job.Name, SourcePath = file, TargetPath = $"{zipFileName}/{entryName}", Status = "ZIPADO" });
                    }
                    catch (Exception ex)
                    {
                        failedPaths.Add(file);
                        auditEntries.Add(new AuditEntry { JobName = job.Name, SourcePath = file, Status = "FALHA", Details = ex.Message });
                    }
                }
                else
                {
                    var fileMeta = fileOperator.GetFileMetadata(file);
                    progressInfo.ProcessedBytes += fileMeta?.Size ?? 0;
                    auditEntries.Add(new AuditEntry { JobName = job.Name, SourcePath = file, Status = "IGNORADO", Details = "Filtro" });
                }

                progressInfo.ProcessedFiles++;
                reportProgress(false);
            }
        }
        catch (OperationCanceledException)
        {
            if (fileOperator.Exists(targetZipPath)) await fileOperator.DeleteAsync(targetZipPath, CancellationToken.None);
            throw;
        }
        catch (Exception ex)
        {
            if (fileOperator.Exists(targetZipPath)) await fileOperator.DeleteAsync(targetZipPath, CancellationToken.None);
            throw new Exception($"Falha ao criar arquivo ZIP: {ex.Message}", ex);
        }
    }

    private async Task ApplyRetentionPolicyAsync(Job job, IFileOperator fileOperator)
    {
        try
        {
            // Pega arquivos no diretrio alvo (apenas no root onde os zips ficam, ou todos baseados no nome do job)
            var targetFiles = fileOperator.EnumerateFiles(job.TargetPath, "*.*", false).ToList();
            var filesInfo = targetFiles.Select(f => new { Path = f, Meta = fileOperator.GetFileMetadata(f) })
                                       .Where(f => f.Meta != null)
                                       .OrderByDescending(f => f.Meta!.LastWriteTimeUtc)
                                       .ToList();

            if (job.RetentionPolicy == RetentionPolicy.KeepLastXVersions && job.RetentionCount > 0)
            {
                var toDelete = filesInfo.Skip(job.RetentionCount).ToList();
                foreach (var file in toDelete)
                {
                    await fileOperator.DeleteAsync(file.Path);
                    await _activityService.LogActivityAsync($"Reteno: Arquivo antigo excludo '{Path.GetFileName(file.Path)}'.", "INFO");
                }
            }
            else if (job.RetentionPolicy == RetentionPolicy.KeepDays && job.RetentionCount > 0)
            {
                var cutoffDate = DateTime.UtcNow.AddDays(-job.RetentionCount);
                var toDelete = filesInfo.Where(f => f.Meta!.LastWriteTimeUtc < cutoffDate).ToList();
                foreach (var file in toDelete)
                {
                    await fileOperator.DeleteAsync(file.Path);
                    await _activityService.LogActivityAsync($"Reteno: Arquivo expirado excludo '{Path.GetFileName(file.Path)}'.", "INFO");
                }
            }
        }
        catch (Exception ex)
        {
            await _logger.LogAsync($"Falha ao aplicar poltica de reteno: {ex.Message}", "WARNING");
        }
    }

    private async Task MoveToTrashAsync(string filePath, Job job, IFileOperator fileOperator, CancellationToken cancellationToken)
    {
        if (!job.EnableTrash) return;
        try
        {
            var trashFolder = Path.Combine(job.TargetPath, ".folderflow", "trash", DateTime.Now.ToString("yyyyMMdd"));
            fileOperator.CreateDirectory(trashFolder);
            await fileOperator.CopyAsync(filePath, Path.Combine(trashFolder, $"{Guid.NewGuid()}_{Path.GetFileName(filePath)}"), cancellationToken);
        }
        catch { }
    }

    private string GenerateUniqueFileName(string filePath, IFileOperator fileOperator)
    {
        var dir = Path.GetDirectoryName(filePath);
        var fileName = Path.GetFileNameWithoutExtension(filePath);
        var ext = Path.GetExtension(filePath);
        var count = 1;
        string newPath;
        do { newPath = Path.Combine(dir ?? string.Empty, $"{fileName}_{count}{ext}"); count++; } while (fileOperator.Exists(newPath));
        return newPath;
    }
}
