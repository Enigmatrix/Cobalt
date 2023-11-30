using Cobalt.Common.Data;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Serilog;

namespace Cobalt.Common.Infrastructure;

public class ServiceInjector
{
    private readonly IServiceProvider _serviceProvider;

#if DEBUG
    private const string AppSettings = "appsettings.Debug.json";
#else
    private const string AppSettings = "appsettings.json";
#endif

    public ServiceInjector(ServiceCollection? services = null)
    {
        // Initialize Configuration
        var configuration = new ConfigurationBuilder().AddJsonFile(AppSettings).Build();

        // Initialize Serilog
        Log.Logger = new LoggerConfiguration().ReadFrom.Configuration(configuration).CreateLogger();

        services ??= new ServiceCollection();
        services
            .AddLogging(logging => logging.AddSerilog(Log.Logger))
            .AddPooledDbContextFactory<QueryContext>(options => QueryContext.ConfigureFor(options,
                configuration.GetConnectionString(nameof(QueryContext)) ??
                throw new InvalidOperationException("Connection string not found")));
        _serviceProvider = services.BuildServiceProvider();
    }

    public T Resolve<T>() where T : notnull
    {
        return _serviceProvider.GetRequiredService<T>();
    }
}