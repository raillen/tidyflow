using System;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
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
        try
        {
            System.IO.File.AppendAllText("trace.txt", "1. Starting DI configuration...\n");
            var services = new ServiceCollection();
            ConfigureServices(services);
            Services = services.BuildServiceProvider();

            System.IO.File.AppendAllText("trace.txt", "2. DI configured.\n");

            if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
            {
                DisableAvaloniaDataAnnotationValidation();
                
                System.IO.File.AppendAllText("trace.txt", "3. Creating MainWindow...\n");
                var mainWindowViewModel = Services.GetRequiredService<MainWindowViewModel>();
                desktop.MainWindow = new MainWindow
                {
                    DataContext = mainWindowViewModel,
                };
                
                desktop.MainWindow.Opened += (s, e) => System.IO.File.AppendAllText("trace.txt", "   -> MainWindow Opened.\n");
                desktop.MainWindow.Closed += (s, e) => System.IO.File.AppendAllText("trace.txt", "   -> MainWindow Closed.\n");

                System.IO.File.AppendAllText("trace.txt", "4. Starting InitializeServicesAsync...\n");
                // Inicialização assíncrona para evitar deadlock na UI
                _ = InitializeServicesAsync();
            }
        }
        catch (Exception ex)
        {
            System.IO.File.WriteAllText("init_error.txt", ex.ToString());
            throw;
        }

        base.OnFrameworkInitializationCompleted();
        System.IO.File.AppendAllText("trace.txt", "5. OnFrameworkInitializationCompleted done.\n");
    }

    public void OnOpenClick(object? sender, EventArgs e)
    {
        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop && desktop.MainWindow is Window window)
        {
            // Força a exibição e restauração do estado correto da janela
            window.Show();
            
            if (window.WindowState == WindowState.Minimized)
            {
                window.WindowState = WindowState.Normal;
            }

            // Hack para trazer a janela para frente em alguns SOs
            window.Topmost = true;
            window.Topmost = false;

            window.Activate();
            window.Focus();
        }
    }

    public void OnExitClick(object? sender, EventArgs e)
    {
        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop && desktop.MainWindow is MainWindow window)
        {
            window.ForceClose();
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
        services.AddSingleton<ISystemActivityService, SystemActivityService>();
        services.AddSingleton<GlobalProgressService>();

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
