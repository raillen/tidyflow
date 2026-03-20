using System;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Data.Core.Plugins;
using Avalonia.Markup.Xaml;
using FolderFlow.App.Services;
using FolderFlow.App.ViewModels;
using FolderFlow.App.Views;
using FolderFlow.Application.Interfaces;
using FolderFlow.Application.Services;
using FolderFlow.Infrastructure.Filesystem;
using FolderFlow.Infrastructure.Localization;
using FolderFlow.Infrastructure.Logging;
using FolderFlow.Infrastructure.Persistence.Json;
using FolderFlow.Infrastructure.Watching;
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
        var services = new ServiceCollection();
        ConfigureServices(services);
        Services = services.BuildServiceProvider();

        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
        {
            DisableAvaloniaDataAnnotationValidation();
            
            var mainWindowViewModel = Services.GetRequiredService<MainWindowViewModel>();
            desktop.MainWindow = new MainWindow
            {
                DataContext = mainWindowViewModel,
            };

            // Inicialização assíncrona para evitar deadlock na UI
            _ = InitializeServicesAsync();
        }

        base.OnFrameworkInitializationCompleted();
    }

    public void OnOpenClick(object? sender, EventArgs e)
    {
        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop && desktop.MainWindow != null)
        {
            desktop.MainWindow.Show();
            desktop.MainWindow.Activate();
        }
    }

    public void OnExitClick(object? sender, EventArgs e)
    {
        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
        {
            desktop.Shutdown();
        }
    }

    private async Task InitializeServicesAsync()
    {
        if (Services == null) return;

        var settingsStore = Services.GetRequiredService<ISettingsStore>();
        var themeService = Services.GetRequiredService<ThemeService>();
        var localizationService = Services.GetRequiredService<ILocalizationService>();
        var queueProcessor = Services.GetRequiredService<QueueProcessor>();
        var schedulerService = Services.GetRequiredService<ISchedulerService>();
        var watchAppService = Services.GetRequiredService<WatchAppService>();

        // Carregar configurações, aplicar tema e idioma
        var settings = await settingsStore.LoadAsync();
        Avalonia.Threading.Dispatcher.UIThread.Post(() => {
            themeService.SetTheme(settings.Theme);
            localizationService.SetLanguage(settings.Language);
        });

        // Iniciar serviços de background
        queueProcessor.Start(System.Threading.CancellationToken.None);
        schedulerService.Start(System.Threading.CancellationToken.None);
        await watchAppService.InitializeAsync();
    }

    private void ConfigureServices(IServiceCollection services)
    {
        // Infrastructure
        services.AddSingleton<ISettingsStore>(new SettingsJsonStore());
        services.AddSingleton<IJobStore>(new JobJsonStore());
        services.AddSingleton<IFileOperator, LocalFileOperator>();
        services.AddSingleton<IWatchService, NativeWatchService>();
        services.AddSingleton<IAppLogger, FileLogger>();
        services.AddSingleton<IAuditService, CsvAuditService>();
        services.AddSingleton<IFailureStore, JsonFailureStore>();
        services.AddSingleton<IJobQueue, ChannelJobQueue>();
        services.AddSingleton<IHashService, Sha256HashService>();
        services.AddSingleton<ICloudHydrationService, WindowsCloudHydrationService>();
        services.AddSingleton<ILocalizationService, JsonLocalizationService>();

        // Application Services
        services.AddSingleton<JobAppService>();
        services.AddSingleton<ExecutionEngine>();
        services.AddSingleton<PreviewEngine>();
        services.AddSingleton<QueueProcessor>();
        services.AddSingleton<WatchAppService>();
        services.AddSingleton<ISchedulerService, SimpleScheduler>();

        // UI Services
        services.AddSingleton<ThemeService>();
        services.AddSingleton<IStorageService, AvaloniaStorageService>();
        services.AddSingleton<INotificationService, AvaloniaNotificationService>();

        // ViewModels
        services.AddTransient<DashboardViewModel>();
        services.AddTransient<JobsViewModel>();
        services.AddTransient<JobEditorViewModel>();
        services.AddTransient<HistoryViewModel>();
        services.AddTransient<SettingsViewModel>();
        services.AddSingleton<MainWindowViewModel>();
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
}
