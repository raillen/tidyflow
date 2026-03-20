using System;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;
using System.Threading;
using System.Threading.Tasks;
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

    public ExecutionEngine(
        IFileOperator fileOperator, 
        IAppLogger logger, 
        IHashService hashService,
        INotificationService notificationService,
        ICloudHydrationService cloudHydrationService,
        IAuditService auditService,
        IFailureStore failureStore)
    {
        _fileOperator = fileOperator;
        _logger = logger;
        _hashService = hashService;
        _notificationService = notificationService;
        _cloudHydrationService = cloudHydrationService;
        _auditService = auditService;
        _failureStore = failureStore;
    }

    public async Task RunJobAsync(Job job, CancellationToken cancellationToken = default, bool isRetry = false, IProgress<JobProgressInfo>? progress = null)
    {
        var auditEntries = new System.Collections.Generic.List<AuditEntry>();
        var failedPaths = new System.Collections.Generic.List<string>();
        var progressInfo = new JobProgressInfo { JobId = job.Id, JobName = job.Name };

        try
        {
            await _logger.LogAsync($"Iniciando Job: {job.Name}" + (isRetry ? " [MODO RETRY]" : ""));

            if (!_fileOperator.Exists(job.SourcePath) && !isRetry)
            {
                var msg = $"Pasta de origem não encontrada: {job.SourcePath}";
                await _logger.LogAsync($"ERRO: {msg}", "ERROR");
                _notificationService.Show("Erro no Job", msg, true);
                auditEntries.Add(new AuditEntry { JobName = job.Name, Status = "FALHA", Details = msg });
                await _auditService.SaveReportAsync(job.Name, auditEntries);
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
            int processedCount = 0;

            foreach (var file in filesToProcess)
            {
                if (cancellationToken.IsCancellationRequested) break;

                progressInfo.CurrentFile = Path.GetFileName(file);
                progress?.Report(progressInfo);

                if (ShouldProcessFile(file, job))
                {
                    // RETRY AUTOMÁTICO (3 vezes)
                    AuditEntry? entry = null;
                    int attempts = 0;
                    while (attempts < 3)
                    {
                        entry = await ProcessFileAsync(file, job, cancellationToken);
                        if (entry != null && entry.Status != "FALHA") break;
                        
                        attempts++;
                        if (attempts < 3) await Task.Delay(1000, cancellationToken);
                    }

                    if (entry != null)
                    {
                        auditEntries.Add(entry);
                        if (entry.Status == "FALHA") failedPaths.Add(file);
                        else if (entry.Status != "IGNORADO") processedCount++;
                    }
                }
                else
                {
                    auditEntries.Add(new AuditEntry { JobName = job.Name, SourcePath = file, Status = "IGNORADO", Details = "Filtro de exclusão ou critérios de data/tamanho." });
                }

                progressInfo.ProcessedFiles++;
                progress?.Report(progressInfo);
            }

            if (failedPaths.Any()) await _failureStore.SaveFailuresAsync(job.Id, failedPaths);
            else await _failureStore.ClearFailuresAsync(job.Id);

            await _auditService.SaveReportAsync(job.Name, auditEntries);
            await _logger.LogAsync($"Job Finalizado: {job.Name}. {processedCount} processados.");
            _notificationService.Show("Job Concluído", $"{job.Name}: {processedCount} processados.");
        }
        catch (Exception ex)
        {
            await _logger.LogAsync($"ERRO CRÍTICO: {ex.Message}", "ERROR");
            _notificationService.Show("Erro Crítico", ex.Message, true);
        }
    }

    private bool ShouldProcessFile(string file, Job job)
    {
        var info = new FileInfo(file);
        
        // Extensões
        if (job.IncludeExtensions.Any())
        {
            var ext = info.Extension.ToLower();
            if (!job.IncludeExtensions.Any(e => e.Trim().ToLower() == ext)) return false;
        }

        // Regex
        if (!string.IsNullOrWhiteSpace(job.NameRegex))
        {
            if (!Regex.IsMatch(info.Name, job.NameRegex, RegexOptions.IgnoreCase)) return false;
        }

        // Exclusão
        if (job.ExcludePatterns.Any(p => file.Contains(p, StringComparison.OrdinalIgnoreCase))) return false;

        // Tamanho
        if (job.MinSizeKB.HasValue && info.Length < job.MinSizeKB * 1024) return false;
        if (job.MaxSizeKB.HasValue && info.Length > job.MaxSizeKB * 1024) return false;

        // Data
        if (job.ModifiedWithinDays.HasValue)
        {
            if (DateTime.Now - info.LastWriteTime > TimeSpan.FromDays(job.ModifiedWithinDays.Value)) return false;
        }

        return true;
    }

    private async Task<AuditEntry?> ProcessFileAsync(string sourceFile, Job job, CancellationToken cancellationToken)
    {
        var entry = new AuditEntry { JobName = job.Name, SourcePath = sourceFile };
        try
        {
            _cloudHydrationService.EnsureFileIsLocal(sourceFile);
            var relativePath = Path.GetRelativePath(job.SourcePath, sourceFile);
            var targetFile = Path.Combine(job.TargetPath, relativePath);
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

            if (job.Mode == JobMode.Copy) await _fileOperator.CopyAsync(sourceFile, targetFile, cancellationToken);
            else await _fileOperator.MoveAsync(sourceFile, targetFile, cancellationToken);

            entry.Status = (job.Mode == JobMode.Copy) ? "COPIADO" : "MOVIDO";
            return entry;
        }
        catch (Exception ex) { entry.Status = "FALHA"; entry.Details = ex.Message; return entry; }
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
