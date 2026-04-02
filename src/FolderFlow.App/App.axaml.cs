using System;
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

    public override async void OnFrameworkInitializationCompleted()
    {
        var serviceCollection = new ServiceCollection();
        ConfigureServices(serviceCollection);
        Services = serviceCollection.BuildServiceProvider();

        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
        {
            // Remove Avalonia data validation so that native DataAnnotations validation will be used
            DisableAvaloniaDataAnnotationValidation();
            
            var mainWindow = new MainWindow();
            desktop.MainWindow = mainWindow;
            
            // Inicializa Servios
            await InitializeServicesAsync();
            
            var mainVm = Services.GetRequiredService<MainWindowViewModel>();
            mainVm.CurrentPage = mainVm.Dashboard; // Inicializa a primeira pgina aqui para evitar loops
            mainWindow.DataContext = mainVm;
        }

        base.OnFrameworkInitializationCompleted();
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
        services.AddSingleton<SettingsViewModel>();
        services.AddTransient<JobEditorViewModel>();
        services.AddTransient<BlueprintEditorViewModel>();
        services.AddTransient<DonateViewModel>();
    }

    private async Task InitializeServicesAsync()
    {
        var settingsStore = Services!.GetRequiredService<ISettingsStore>();
        var settings = await settingsStore.LoadAsync();
        
        var localizationService = Services!.GetRequiredService<ILocalizationService>();
        localizationService.SetLanguage(settings.Language);

        var themeService = Services!.GetRequiredService<ThemeService>();
        themeService.SetTheme(settings.Theme);

        var watchAppService = Services!.GetRequiredService<WatchAppService>();
        await watchAppService.InitializeAsync();

        var scheduler = Services!.GetRequiredService<ISchedulerService>();
        scheduler.Start(default);
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
            desktop.MainWindow?.Show();
            desktop.MainWindow?.Activate();
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
