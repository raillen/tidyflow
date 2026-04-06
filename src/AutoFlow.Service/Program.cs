using System;
using AutoFlow.Application.Interfaces;
using AutoFlow.Application.Services;
using AutoFlow.Infrastructure.Execution;
using AutoFlow.Infrastructure.Filesystem;
using AutoFlow.Infrastructure.Localization;
using AutoFlow.Infrastructure.Logging;
using AutoFlow.Infrastructure.Notifications;
using AutoFlow.Infrastructure.Persistence.Json;
using AutoFlow.Infrastructure.Security;
using AutoFlow.Infrastructure.Watching;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;

namespace AutoFlow.Service;

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
                options.ServiceName = "AutoFlow Daemon";
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
                services.AddSingleton<IJobQueue, ObservableJobQueue>();
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
