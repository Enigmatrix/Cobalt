using System.Windows.Threading;
using Cobalt.Common.UI;
using Cobalt.ViewModels;
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

    public class AppBoostrapper : Bootstrapper<IMainViewModel>
    {
        protected override void PrepareApplication()
        {
            Log.Logger = new LoggerConfiguration()
                .WriteTo.File("./main-log.txt")
                .CreateLogger();
            Log.Information("NEW SESSION");
            base.PrepareApplication();
        }

        protected override void OnUnhandledException(object sender, DispatcherUnhandledExceptionEventArgs e)
        {
            Log.Information($"Exception raised in Cobalt: {e.Exception}");
        }
    }
}