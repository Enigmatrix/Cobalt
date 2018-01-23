using System.Windows.Threading;
using Autofac;
using Cobalt.Common.UI;
using Cobalt.ViewModels;
using Cobalt.ViewModels.Utils;
using Serilog;

namespace Cobalt
{
    /// <inheritdoc />
    /// <summary>
    ///     Interaction logic for App.xaml
    /// </summary>
    public partial class CobaltApp
    {
    }

    public class AppBoostrapper : Bootstrapper<MainViewModel>
    {
        protected override void PrepareApplication()
        {
            Log.Logger = new LoggerConfiguration()
                .WriteTo.File("./main-log.txt")
                .CreateLogger();
            Log.Information("NEW SESSION");
            base.PrepareApplication();
        }

        protected override void ConfigureContainer(ContainerBuilder builder)
        {
            base.ConfigureContainer(builder);
            builder.RegisterType<NavigationService>()
                .As<INavigationService>()
                .SingleInstance();
        }

        protected override void OnUnhandledException(object sender, DispatcherUnhandledExceptionEventArgs e)
        {
            Log.Information($"Exception raised in Cobalt: {e.Exception}");
        }
    }
}