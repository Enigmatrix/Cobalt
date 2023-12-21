using Cobalt.Common.Data;
using Cobalt.Common.ViewModels;
using Cobalt.Common.ViewModels.Entities;
using Cobalt.Common.ViewModels.Pages;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Serilog;

namespace Cobalt.Common.Infrastructure;

/// <summary>
///     IoC Service Root
/// </summary>
public class ServiceInjector
{
    private const string DebugAppSettings = "dbg/appsettings.Debug.json";
    private const string AppSettings = "appsettings.json";
    private readonly IServiceProvider _serviceProvider;

    /// <summary>
    ///     Initialize the IoC Service Root
    /// </summary>
    /// <param name="services">Pre-initialized <see cref="ServiceCollection" /></param>
    /// <exception cref="InvalidOperationException">When configuration variables are not found</exception>
    public ServiceInjector(ServiceCollection? services = null)
    {
        // Initialize Configuration. Merge the normal AppSettings with the DebugAppSettings (in debug mode).
        var configuration = new ConfigurationBuilder()
            .AddJsonFile(AppSettings)
#if DEBUG
            .AddJsonFile(DebugAppSettings)
#endif
            .Build();

        // Initialize Serilog. Read options from the Configuration.
        Log.Logger = new LoggerConfiguration().ReadFrom.Configuration(configuration).CreateLogger();

        // Register Services
        services ??= new ServiceCollection();
        services
            .AddLogging(logging => logging.AddSerilog(Log.Logger))
            .AddPooledDbContextFactory<QueryContext>(options => QueryContext.ConfigureFor(options,
                configuration.GetConnectionString(nameof(QueryContext)) ??
                throw new InvalidOperationException("Connection string not found")));

        services.AddSingleton<IEntityViewModelCache, EntityViewModelCache>();

        // Register ViewModels
        RegisterViewModels(services);

        _serviceProvider = services.BuildServiceProvider();
    }

    /// <summary>
    ///     Register the ViewModels involved
    /// </summary>
    public static void RegisterViewModels(ServiceCollection services)
    {
        services.AddSingleton<MainViewModel>();

        // Pages
#if DEBUG
        services.AddSingleton<ExperimentsPageViewModel>();
#endif
        services.AddSingleton<HomePageViewModel>();
        services.AddSingleton<AppsPageViewModel>();
        services.AddSingleton<TagsPageViewModel>();
        services.AddSingleton<AlertsPageViewModel>();
        services.AddSingleton<HistoryPageViewModel>();
    }

    /// <summary>
    ///     Resolve a dependency
    /// </summary>
    /// <typeparam name="T">Type of the dependency</typeparam>
    /// <returns></returns>
    public T Resolve<T>() where T : notnull
    {
        return _serviceProvider.GetRequiredService<T>();
    }
}