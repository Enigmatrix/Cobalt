using System;
using System.Windows;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using ReactiveUI;
using Splat;
using Splat.Microsoft.Extensions.DependencyInjection;
using Splat.Microsoft.Extensions.Logging;
using MsHost = Microsoft.Extensions.Hosting.Host;

namespace Cobalt.Common.Views
{
    public class CobaltApp : Application
    {
        protected readonly IHost Host;

        public CobaltApp()
        {
            Host = MsHost.CreateDefaultBuilder()
                .ConfigureServices((ctx, svcs) =>
                {
                    svcs.UseMicrosoftDependencyResolver();
                    var resolver = Locator.CurrentMutable;
                    resolver.InitializeSplat();
                    resolver.InitializeReactiveUI();

                    ConfigureServices(ctx.Configuration, svcs);
                })
                .ConfigureLogging(log => { log.AddSplat(); })
                .Build();
        }

        protected virtual void ConfigureServices(IConfiguration configuration,
            IServiceCollection services)
        {
        }

        protected override async void OnStartup(StartupEventArgs e)
        {
            await Host.StartAsync();
            HostStartup(e);
            base.OnStartup(e);
        }

        protected virtual void HostStartup(StartupEventArgs e)
        {
        }

        protected override async void OnExit(ExitEventArgs e)
        {
            await Host.StopAsync(TimeSpan.FromSeconds(5));
            Host.Dispose();
            base.OnExit(e);
        }
    }
}