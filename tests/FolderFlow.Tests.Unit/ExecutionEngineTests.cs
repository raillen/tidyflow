using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using FolderFlow.Application.Interfaces;
using FolderFlow.Application.Services;
using FolderFlow.Domain.Entities;
using FolderFlow.Domain.Enums;
using FolderFlow.Domain.ValueObjects;
using Xunit;

namespace FolderFlow.Tests.Unit;

public class ExecutionEngineTests
{
    private class MockFileOperator : IFileOperator
    {
        public List<string> CreatedDirectories = new();
        public List<(string Source, string Target)> CopiedFiles = new();
        public List<string> DeletedFiles = new();
        public List<string> Files = new();

        public Task CopyAsync(string source, string target, CancellationToken cancellationToken = default, IProgress<double>? progress = null)
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

    [Fact]
    public async Task RunJobAsync_ShouldCopyFiles_WhenFiltersMatch()
    {
        // Arrange
        var sourceDir = Path.Combine("C:", "Source");
        var targetDir = Path.Combine("D:", "Target");
        var sourceFile = Path.Combine(sourceDir, "file1.txt");
        var targetFile = Path.Combine(targetDir, "file1.txt");

        var fileOp = new MockFileOperator();
        fileOp.Files.Add(sourceFile);
        fileOp.Files.Add(Path.Combine(sourceDir, "file2.pdf"));
        fileOp.CreatedDirectories.Add(sourceDir);

        var logger = new MockLogger();
        var hashService = new MockHashService();
        var notificationService = new MockNotificationService();
        var cloudService = new MockCloudHydrationService();
        var auditService = new MockAuditService();
        var failureStore = new MockFailureStore();
        var activityService = new MockActivityService();
        var globalProgressService = new GlobalProgressService();
        
        var engine = new ExecutionEngine(fileOp, logger, hashService, notificationService, cloudService, auditService, failureStore, activityService, globalProgressService);

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
