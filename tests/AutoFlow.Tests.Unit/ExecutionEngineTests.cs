using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using AutoFlow.Application.Interfaces;
using AutoFlow.Application.Services;
using AutoFlow.Domain.Entities;
using AutoFlow.Domain.Enums;
using AutoFlow.Domain.ValueObjects;
using Xunit;

namespace AutoFlow.Tests.Unit;

public class ExecutionEngineTests
{
    private class LocalFileOperator : IFileOperator
    {
        public List<string> CreatedDirectories = new();
        public List<(string Source, string Target)> CopiedFiles = new();
        public List<string> DeletedFiles = new();
        public List<string> Files = new();

        public long BandwidthLimit { get; set; }
        public Task CopyAsync(string source, string target, CancellationToken cancellationToken = default, IProgress<double>? progress = null, string? encryptionKey = null, bool deltaSync = false)
        {
            CopiedFiles.Add((source, target));
            return Task.CompletedTask;
        }

        public Task MoveAsync(string source, string target, CancellationToken cancellationToken = default) => Task.CompletedTask;
        public Task DeleteAsync(string path, CancellationToken cancellationToken = default)
        {
            DeletedFiles.Add(path);
            return Task.CompletedTask;
        }
        public bool Exists(string path) => Files.Contains(path) || CreatedDirectories.Contains(path);
        public IEnumerable<string> EnumerateFiles(string path, string searchPattern, bool recursive) => Files.Where(f => f.StartsWith(path));
        public void CreateDirectory(string path) => CreatedDirectories.Add(path);
        public FileMetadata? GetFileMetadata(string path) => Files.Contains(path) ? new FileMetadata(100, System.DateTime.UtcNow) : null;
    }

    private class MockLogger : IAppLogger
    {
        public Task LogAsync(string message, string level = "INFO") => Task.CompletedTask;
    }

    private class MockHashService : IHashService
    {
        public Task<string> ComputeHashAsync(string filePath) => Task.FromResult("fake-hash");
    }

    private class MockNotificationService : INotificationService
    {
        public void Show(string title, string message, bool isError = false) { }
    }

    private class MockCloudHydrationService : ICloudHydrationService
    {
        public Task EnsureFileIsLocalAsync(string filePath, CancellationToken cancellationToken = default) => Task.CompletedTask;
    }

    private class MockAuditService : IAuditService
    {
        public Task SaveReportAsync(string jobName, IEnumerable<AuditEntry> entries) => Task.CompletedTask;
        public Task<int> PurgeOldLogsAsync(int days) => Task.FromResult(0);
        public Task<string> GetDailySummaryAsync() => Task.FromResult(string.Empty);
    }

    private class MockFailureStore : IFailureStore
    {
        public Task SaveFailuresAsync(Guid jobId, IEnumerable<string> failedPaths) => Task.CompletedTask;
        public Task<IEnumerable<string>> GetFailuresAsync(Guid jobId) => Task.FromResult(Enumerable.Empty<string>());
        public Task ClearFailuresAsync(Guid jobId) => Task.CompletedTask;
    }

    private class MockActivityService : ISystemActivityService
    {
        public Task LogActivityAsync(string message, string level = "INFO") => Task.CompletedTask;
        public Task<IEnumerable<SystemActivity>> GetRecentActivitiesAsync(int count = 50) => Task.FromResult(Enumerable.Empty<SystemActivity>());
    }

    private class MockExternalNotificationService : IExternalNotificationService
    {
        public Task NotifyJobCompletionAsync(Job job, bool success, int processedFiles, string? errorMessage = null) => Task.CompletedTask;
    }

    private class MockScriptRunner : IScriptRunner
    {
        public Task<bool> RunScriptAsync(string scriptPath, CancellationToken cancellationToken = default) => Task.FromResult(true);
    }

    private class MockEncryptionService : IEncryptionService
    {
        public Stream GetEncryptStream(Stream targetStream, string password) => targetStream;
    }

    private class MockSettingsStore : ISettingsStore
    {
        public Task<AppSettings> LoadAsync() => Task.FromResult(new AppSettings());
        public AppSettings Load() => new AppSettings();
        public Task SaveAsync(AppSettings settings) => Task.CompletedTask;
    }


    private class MockLocalizationService : ILocalizationService
    {
        public string this[string key] => key;
        public string GetString(string key) => key;
        public void SetLanguage(string cultureCode) { }
    }

    [Fact]
    public async Task RunJobAsync_ShouldCopyFiles_WhenFiltersMatch()
    {
        // Arrange
        var sourceDir = Path.Combine("C:", "Source");
        var targetDir = Path.Combine("D:", "Target");
        var sourceFile = Path.Combine(sourceDir, "file1.txt");
        var targetFile = Path.Combine(targetDir, "file1.txt");

        var fileOp = new LocalFileOperator();
        fileOp.Files.Add(sourceFile);
        fileOp.Files.Add(Path.Combine(sourceDir, "file2.pdf"));
        fileOp.CreatedDirectories.Add(sourceDir);

        var fileOperatorFactory = new FileOperatorFactory(new IFileOperator[] { fileOp });

        var logger = new MockLogger();
        var hashService = new MockHashService();
        var notificationService = new MockNotificationService();
        var cloudService = new MockCloudHydrationService();
        var auditService = new MockAuditService();
        var failureStore = new MockFailureStore();
        var activityService = new MockActivityService();
        var globalProgressService = new GlobalProgressService();
        var extNotificationService = new MockExternalNotificationService();
        var scriptRunner = new MockScriptRunner();
        var encryptionService = new MockEncryptionService();
        var settingsStore = new MockSettingsStore();
        var localizationService = new MockLocalizationService();
        
        var engine = new ExecutionEngine(
            fileOperatorFactory, 
            logger, 
            hashService, 
            notificationService, 
            cloudService, 
            auditService, 
            failureStore, 
            activityService, 
            globalProgressService, 
            extNotificationService, 
            scriptRunner, 
            encryptionService,
            settingsStore,
            localizationService);

        var job = new Job
        {
            SourcePath = sourceDir,
            TargetPath = targetDir,
            Mode = JobMode.Copy,
            IncludeExtensions = new List<string> { ".txt" }
        };

        // Act
        await engine.RunJobAsync(job);

        // Assert
        Assert.Single(fileOp.CopiedFiles);
        Assert.Equal(sourceFile, fileOp.CopiedFiles[0].Source);
        Assert.Equal(targetFile, fileOp.CopiedFiles[0].Target);
    }
}
