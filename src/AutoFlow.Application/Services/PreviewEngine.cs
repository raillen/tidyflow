using System;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;
using System.Threading;
using System.Threading.Tasks;
using AutoFlow.Application.Interfaces;
using AutoFlow.Domain.Entities;
using AutoFlow.Domain.Enums;
using AutoFlow.Domain.ValueObjects;

namespace AutoFlow.Application.Services;

public class PreviewEngine
{
    private readonly IFileOperator _fileOperator;
    private readonly ILocalizationService _localizationService;
    private readonly IMetadataService _metadataService;

    public PreviewEngine(IFileOperator fileOperator, ILocalizationService localizationService, IMetadataService metadataService)
    {
        _fileOperator = fileOperator;
        _localizationService = localizationService;
        _metadataService = metadataService;
    }

    public async Task<PreviewSummary> GeneratePreviewAsync(Job job, CancellationToken cancellationToken = default)
    {
        return await Task.Run(() =>
        {
            var summary = new PreviewSummary();

            if (!_fileOperator.Exists(job.SourcePath))
            {
                summary.AffectedPaths.Add(string.Format(_localizationService["PreviewErrorSourceNotFound"], job.SourcePath));
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
                    summary.AffectedPaths.Add(string.Format(_localizationService["PreviewIgnored"], Path.GetFileName(file)));
                }
            }

            return summary;
        }, cancellationToken);
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

        // Filtros Avançados: Conteúdo
        if (!string.IsNullOrWhiteSpace(job.ContentContains))
        {
            if (!_metadataService.ContainsText(file, job.ContentContains)) return false;
        }

        // Filtros Avançados: EXIF Data
        if (job.ExifDateStart.HasValue || job.ExifDateEnd.HasValue)
        {
            var exifDate = _metadataService.GetExifDate(file);
            if (exifDate == null) return false;

            if (job.ExifDateStart.HasValue && exifDate.Value < job.ExifDateStart.Value) return false;
            if (job.ExifDateEnd.HasValue && exifDate.Value > job.ExifDateEnd.Value) return false;
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
                    summary.AffectedPaths.Add(string.Format(_localizationService["PreviewIgnoredSmartSync"], relativePath));
                    return;
                }
            }

            switch (job.ConflictMode)
            {
                case ConflictMode.Skip:
                    summary.FilesToSkip++;
                    summary.AffectedPaths.Add(string.Format(_localizationService["PreviewIgnoredConflict"], relativePath));
                    break;
                case ConflictMode.Overwrite:
                    summary.FilesToOverwrite++;
                    summary.TotalBytesToTransfer += fileSize;
                    summary.AffectedPaths.Add(string.Format(_localizationService["PreviewOverwrite"], relativePath));
                    break;
                case ConflictMode.Rename:
                    summary.TotalBytesToTransfer += fileSize;
                    summary.AffectedPaths.Add(string.Format(_localizationService["PreviewRename"], relativePath));
                    break;
            }
        }
        else
        {
            if (job.Mode == JobMode.Copy)
            {
                summary.FilesToCopy++;
                summary.TotalBytesToTransfer += fileSize;
                summary.AffectedPaths.Add(string.Format(_localizationService["PreviewCopy"], relativePath));
            }
            else
            {
                summary.FilesToMove++;
                summary.TotalBytesToTransfer += fileSize;
                summary.AffectedPaths.Add(string.Format(_localizationService["PreviewMove"], relativePath));
            }
        }
    }
}
