using Cobalt.Common.Data;
using Cobalt.Common.ViewModels.Entities;
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
            .AddSingleton<IEntityViewModelCache, EntityViewModelCache>();
    }

    public IServiceProvider Build()
    {
        return _serviceColl.BuildServiceProvider();
    }
}