using System;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Data.Core.Plugins;
using Avalonia.Markup.Xaml;
using FolderFlow.App.ViewModels;
using FolderFlow.App.Views;
using FolderFlow.App.Services;
using FolderFlow.Application.Interfaces;
using FolderFlow.Application.Services;
using FolderFlow.Infrastructure.Localization;
using FolderFlow.Infrastructure.Logging;
using FolderFlow.Infrastructure.Persistence;
using FolderFlow.Infrastructure.Persistence.Json;
using FolderFlow.Infrastructure.Security;
using FolderFlow.Infrastructure.Execution;
using FolderFlow.Infrastructure.Filesystem;
using FolderFlow.Infrastructure.Watching;
using FolderFlow.Infrastructure.Notifications;
using Microsoft.Extensions.DependencyInjection;

namespace FolderFlow.App;

public partial class App : Avalonia.Application
{
    public static IServiceProvider? Services { get; private set; }

    public override void Initialize()
    {
        AvaloniaXamlLoader.Load(this);
    }

    public override void OnFrameworkInitializationCompleted()
    {
        try 
        {
            var serviceCollection = new ServiceCollection();
            ConfigureServices(serviceCollection);
            Services = serviceCollection.BuildServiceProvider();

            if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
            {
                // Carrega configurações sincronamente para o startup
                var settingsStore = Services.GetRequiredService<ISettingsStore>();
                var settings = settingsStore.Load();
                
                var localizationService = Services.GetRequiredService<ILocalizationService>();
                localizationService.SetLanguage(settings.Language);

                var themeService = Services.GetRequiredService<ThemeService>();
                themeService.SetTheme(settings.Theme);

                var mainVm = Services.GetRequiredService<MainWindowViewModel>();
                mainVm.CurrentPage = mainVm.Dashboard; 

                var mainWindow = new MainWindow
                {
                    DataContext = mainVm,
                    WindowState = WindowState.Normal
                };
                desktop.MainWindow = mainWindow;
                mainWindow.Show();

                // Inicializa serviços pesados em background
                _ = Task.Run(async () => {
                    try 
                    {
                        var watchAppService = Services.GetRequiredService<WatchAppService>();
                        await watchAppService.InitializeAsync();

                        var scheduler = Services.GetRequiredService<ISchedulerService>();
                        scheduler.Start(default);
                    }
                    catch { /* Log silently or handle background failure */ }
                });
            }

            base.OnFrameworkInitializationCompleted();
        }
        catch (Exception ex)
        {
            File.WriteAllText("crash_log.txt", $"CRASH AT STARTUP: {ex.Message}\n{ex.StackTrace}");
            if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
            {
                desktop.Shutdown();
            }
        }
    }

    private void ConfigureServices(IServiceCollection services)
    {
        // Infrastructure
        services.AddSingleton<IAppLogger, FileLogger>();
        services.AddSingleton<IAuditService, SqliteAuditService>();
        services.AddSingleton<ISystemActivityService, SystemActivityService>();
        services.AddSingleton<ILocalizationService, JsonLocalizationService>();
        services.AddSingleton<ISettingsStore, SettingsJsonStore>();
        services.AddSingleton<IJobStore, JobJsonStore>();
        services.AddSingleton<IFailureStore, JsonFailureStore>();
        services.AddSingleton<IBlueprintStore, JsonBlueprintStore>();
        services.AddSingleton<IHashService, Sha256HashService>();
        services.AddSingleton<IEncryptionService, EncryptionService>();
        services.AddSingleton<IFileOperator, LocalFileOperator>();
        services.AddSingleton<FileOperatorFactory>();
        services.AddSingleton<IScriptRunner, LocalScriptRunner>();
        services.AddSingleton<IWatchService, NativeWatchService>();
        services.AddSingleton<INotificationService, AvaloniaNotificationService>();
        services.AddSingleton<IExternalNotificationService, WebhookNotificationService>();
        services.AddSingleton<ICloudHydrationService, WindowsCloudHydrationService>();
        services.AddSingleton<IStorageService, AvaloniaStorageService>();
        services.AddSingleton<IJobQueue, ObservableJobQueue>();
        services.AddSingleton<ISchedulerService, SimpleScheduler>();
        services.AddSingleton<ThemeService>();

        // Application
        services.AddSingleton<JobAppService>();
        services.AddSingleton<BlueprintAppService>();
        services.AddSingleton<ExecutionEngine>();
        services.AddSingleton<PreviewEngine>();
        services.AddSingleton<IOrganizationService, OrganizationService>();
        services.AddSingleton<GlobalProgressService>();
        services.AddSingleton<QueueProcessor>();
        services.AddSingleton<WatchAppService>();

        // Presentation
        services.AddSingleton<MainWindowViewModel>();
        services.AddSingleton<DashboardViewModel>();
        services.AddSingleton<AutomationViewModel>();
        services.AddSingleton<BlueprintViewModel>();
        services.AddSingleton<HistoryViewModel>();
        services.AddSingleton<SettingsViewModel>(sp => new SettingsViewModel(
            sp.GetRequiredService<ISettingsStore>(),
            sp.GetRequiredService<ThemeService>(),
            sp.GetRequiredService<ILocalizationService>(),
            sp.GetRequiredService<IAuditService>(),
            sp.GetRequiredService<INotificationService>()));
        services.AddTransient<JobEditorViewModel>();
        services.AddTransient<BlueprintEditorViewModel>();
        services.AddTransient<DonateViewModel>();
    }

    private void DisableAvaloniaDataAnnotationValidation()
    {
        var dataValidationPluginsToRemove =
            BindingPlugins.DataValidators.OfType<DataAnnotationsValidationPlugin>().ToArray();

        foreach (var plugin in dataValidationPluginsToRemove)
        {
            BindingPlugins.DataValidators.Remove(plugin);
        }
    }

    public void OnOpenClick(object? sender, EventArgs e)
    {
        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
        {
            if (desktop.MainWindow != null)
            {
                desktop.MainWindow.Show();
                desktop.MainWindow.WindowState = WindowState.Normal;
                desktop.MainWindow.Activate();
            }
        }
    }

    public void OnExitClick(object? sender, EventArgs e)
    {
        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
        {
            desktop.Shutdown();
        }
    }
}
