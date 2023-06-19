using Cobalt.Common.Data;
using Cobalt.Common.ViewModels;
using Cobalt.Common.ViewModels.Dialogs;
using Cobalt.Common.ViewModels.Entities;
using Cobalt.Common.ViewModels.Pages;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;

namespace Cobalt.Common.Infrastructure;

public class ServiceManager
{
    private static IServiceProvider? _services;
    private readonly IServiceCollection _serviceColl;

    public ServiceManager()
    {
        _serviceColl = new ServiceCollection();
    }

    public static IServiceProvider Services
    {
        get
        {
            if (_services != null) return _services!;

            var mgr = new ServiceManager();
            mgr.ConfigureServices();
            return _services = mgr.Build();
        }
    }

    public void ConfigureServices()
    {
        // TODO using CurrentDirectory should only be used for Debugging!
        var workDir = Environment.CurrentDirectory;
        var configBuilder = new ConfigurationBuilder().SetBasePath(workDir).AddJsonFile("appsettings.json");
        var config = configBuilder.Build();

        _serviceColl
            .AddSingleton<IConfiguration>(config)
            .AddDbContextFactory<CobaltContext>()
            .AddSingleton<IEntityViewModelCache, EntityViewModelCache>()

            // Dialogs
            .AddTransientWithFactory<AddAlertDialogViewModel>()
            .AddTransientWithFactory<AddTagDialogViewModel>()

            // Pages
            .AddSingleton<AlertsPageViewModel>()
            .AddSingleton<AppsPageViewModel>()
            .AddSingleton<HistoryPageViewModel>()
            .AddSingleton<HomePageViewModel>()
            .AddSingleton<SettingsPageViewModel>()
            .AddSingleton<TagsPageViewModel>()
            .AddSingleton<MainWindowViewModel>();
    }

    public IServiceProvider Build()
    {
        return _serviceColl.BuildServiceProvider();
    }
}

public static class ServiceCollectionExtensions
{
    public static IServiceCollection AddTransientWithFactory<T>(this IServiceCollection coll)
        where T : class
    {
        return coll
            .AddTransient<T>()
            .AddSingleton<Func<T>>(prov => prov.GetRequiredService<T>);
    }
}