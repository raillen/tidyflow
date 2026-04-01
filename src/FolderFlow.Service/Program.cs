using System;
using FolderFlow.Application.Interfaces;
using FolderFlow.Application.Services;
using FolderFlow.Infrastructure.Execution;
using FolderFlow.Infrastructure.Filesystem;
using FolderFlow.Infrastructure.Localization;
using FolderFlow.Infrastructure.Logging;
using FolderFlow.Infrastructure.Notifications;
using FolderFlow.Infrastructure.Persistence.Json;
using FolderFlow.Infrastructure.Security;
using FolderFlow.Infrastructure.Watching;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;

namespace FolderFlow.Service;

public class Program
{
    public static void Main(string[] args)
    {
        CreateHostBuilder(args).Build().Run();
    }

    public static IHostBuilder CreateHostBuilder(string[] args) =>
        Host.CreateDefaultBuilder(args)
            .UseWindowsService(options =>
            {
                options.ServiceName = "FolderFlow Daemon";
            })
            .ConfigureServices((hostContext, services) =>
            {
                // Infrastructure
                services.AddSingleton<ISettingsStore, SettingsJsonStore>();
                services.AddSingleton<IJobStore, JobJsonStore>();
                
                // File Operators setup
                services.AddSingleton<LocalFileOperator>();
                services.AddSingleton<SftpFileOperator>(provider => new SftpFileOperator("localhost", 22, "root", "")); // Mock config, real config via settings
                services.AddSingleton<IFileOperator, LocalFileOperator>(); // Default
                services.AddSingleton(provider => new FileOperatorFactory(new IFileOperator[] { 
                    provider.GetRequiredService<LocalFileOperator>(),
                    provider.GetRequiredService<SftpFileOperator>()
                }));

                services.AddSingleton<IWatchService, NativeWatchService>();
                services.AddSingleton<IAppLogger, FileLogger>();
                services.AddSingleton<IAuditService, CsvAuditService>();
                services.AddSingleton<IFailureStore, JsonFailureStore>();
                services.AddSingleton<IJobQueue, ChannelJobQueue>();
                services.AddSingleton<IHashService, Sha256HashService>();
                services.AddSingleton<ICloudHydrationService, WindowsCloudHydrationService>();
                services.AddSingleton<ILocalizationService, JsonLocalizationService>();
                services.AddSingleton<ISystemActivityService, SystemActivityService>();
                services.AddSingleton<GlobalProgressService>();
                services.AddSingleton<IExternalNotificationService, WebhookNotificationService>();
                services.AddSingleton<IScriptRunner, LocalScriptRunner>();
                services.AddSingleton<IEncryptionService, EncryptionService>();

                // Mock Notification Service for headless service
                services.AddSingleton<INotificationService, HeadlessNotificationService>();

                // Application Services
                services.AddSingleton<JobAppService>();
                services.AddSingleton<ExecutionEngine>();
                services.AddSingleton<QueueProcessor>();
                services.AddSingleton<WatchAppService>();
                services.AddSingleton<ISchedulerService, SimpleScheduler>();

                // The Worker
                services.AddHostedService<Worker>();
            });
}

// Dummy notification service for background daemon where UI is not present
public class HeadlessNotificationService : INotificationService
{
    private readonly IAppLogger _logger;
    public HeadlessNotificationService(IAppLogger logger) => _logger = logger;
    public void Show(string title, string message, bool isError = false) 
    {
        // Suppress or log it instead of UI popup
        _logger.LogAsync($"Headless Alert - {title}: {message}", isError ? "ERROR" : "INFO").Wait();
    }
}
