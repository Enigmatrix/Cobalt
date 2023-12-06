using Cobalt.Common.Data;
using Cobalt.Common.ViewModels;
using Cobalt.Common.ViewModels.Pages;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Serilog;

namespace Cobalt.Common.Infrastructure;

public class ServiceInjector
{
    private const string DebugAppSettings = "appsettings.Debug.json";
    private const string AppSettings = "appsettings.json";
    private readonly IServiceProvider _serviceProvider;

    public ServiceInjector(ServiceCollection? services = null)
    {
        // Initialize Configuration
        var configuration = new ConfigurationBuilder()
            .AddJsonFile(AppSettings)
#if DEBUG
            .AddJsonFile(DebugAppSettings)
#endif
            .Build();

        // Initialize Serilog
        Log.Logger = new LoggerConfiguration().ReadFrom.Configuration(configuration).CreateLogger();

        services ??= new ServiceCollection();
        services
            .AddLogging(logging => logging.AddSerilog(Log.Logger))
            .AddPooledDbContextFactory<QueryContext>(options => QueryContext.ConfigureFor(options,
                configuration.GetConnectionString(nameof(QueryContext)) ??
                throw new InvalidOperationException("Connection string not found")));
        RegisterViewModels(services);
        _serviceProvider = services.BuildServiceProvider();
    }

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

    public T Resolve<T>() where T : notnull
    {
        return _serviceProvider.GetRequiredService<T>();
    }
}