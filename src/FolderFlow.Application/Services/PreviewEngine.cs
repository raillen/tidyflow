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

public class PreviewEngine
{
    private readonly IFileOperator _fileOperator;

    public PreviewEngine(IFileOperator fileOperator)
    {
        _fileOperator = fileOperator;
    }

    public async Task<PreviewSummary> GeneratePreviewAsync(Job job, CancellationToken cancellationToken = default)
    {
        var summary = new PreviewSummary();

        if (!_fileOperator.Exists(job.SourcePath))
        {
            summary.AffectedPaths.Add($"[ERRO] Diretório de origem não encontrado: {job.SourcePath}");
            return summary;
        }

        var files = _fileOperator.EnumerateFiles(job.SourcePath, "*", job.Recursive).ToList();

        foreach (var file in files)
        {
            if (cancellationToken.IsCancellationRequested) break;

            if (ShouldProcessFile(file, job))
            {
                AnalyzeFile(file, job, summary);
            }
            else
            {
                summary.FilesToSkip++;
                summary.AffectedPaths.Add($"[IGNORADO] {Path.GetFileName(file)}");
            }
        }

        return await Task.FromResult(summary);
    }

    private bool ShouldProcessFile(string file, Job job)
    {
        var info = new FileInfo(file);
        
        if (job.IncludeExtensions.Any())
        {
            var ext = info.Extension.ToLower();
            if (!job.IncludeExtensions.Any(e => e.Trim().ToLower() == ext)) return false;
        }

        if (!string.IsNullOrWhiteSpace(job.NameRegex))
        {
            if (!Regex.IsMatch(info.Name, job.NameRegex, RegexOptions.IgnoreCase)) return false;
        }

        if (job.ExcludePatterns.Any(p => file.Contains(p, StringComparison.OrdinalIgnoreCase))) return false;

        if (job.MinSizeKB.HasValue && info.Length < job.MinSizeKB * 1024) return false;
        if (job.MaxSizeKB.HasValue && info.Length > job.MaxSizeKB * 1024) return false;

        if (job.ModifiedWithinDays.HasValue)
        {
            if (DateTime.Now - info.LastWriteTime > TimeSpan.FromDays(job.ModifiedWithinDays.Value)) return false;
        }

        return true;
    }

    private void AnalyzeFile(string sourceFile, Job job, PreviewSummary summary)
    {
        var relativePath = Path.GetRelativePath(job.SourcePath, sourceFile);
        var targetFile = Path.Combine(job.TargetPath, relativePath);
        
        var meta = _fileOperator.GetFileMetadata(sourceFile);
        var fileSize = meta?.Size ?? 0;

        if (_fileOperator.Exists(targetFile))
        {
            if (job.SmartSync)
            {
                var targetMeta = _fileOperator.GetFileMetadata(targetFile);
                if (meta != null && targetMeta != null &&
                    meta.Size == targetMeta.Size &&
                    Math.Abs((meta.LastWriteTimeUtc - targetMeta.LastWriteTimeUtc).TotalSeconds) < 2)
                {
                    summary.FilesToSkip++;
                    summary.AffectedPaths.Add($"[IGNORADO - SMART SYNC] {relativePath}");
                    return;
                }
            }

            switch (job.ConflictMode)
            {
                case ConflictMode.Skip:
                    summary.FilesToSkip++;
                    summary.AffectedPaths.Add($"[IGNORADO - CONFLITO] {relativePath}");
                    break;
                case ConflictMode.Overwrite:
                    summary.FilesToOverwrite++;
                    summary.TotalBytesToTransfer += fileSize;
                    summary.AffectedPaths.Add($"[SOBRESCREVER] {relativePath}");
                    break;
                case ConflictMode.Rename:
                    summary.TotalBytesToTransfer += fileSize;
                    summary.AffectedPaths.Add($"[RENOMEAR] {relativePath}");
                    break;
            }
        }
        else
        {
            if (job.Mode == JobMode.Copy)
            {
                summary.FilesToCopy++;
                summary.TotalBytesToTransfer += fileSize;
                summary.AffectedPaths.Add($"[COPIAR] {relativePath}");
            }
            else
            {
                summary.FilesToMove++;
                summary.TotalBytesToTransfer += fileSize;
                summary.AffectedPaths.Add($"[MOVER] {relativePath}");
            }
        }
    }
}
