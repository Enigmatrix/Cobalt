using System.Reflection;
using System.Windows;
using System.Windows.Threading;
using Cobalt.Common.IoC;
using Serilog;
using Splat;

namespace Cobalt.Common.UI
{
    public class CobaltApplication : Application
    {
        public CobaltApplication()
        {
            IoCService.Instance = new IoCService(x =>
                x.RegisterForReactiveUI(Assembly.GetEntryAssembly()));
            Locator.CurrentMutable =
                new AutofacDependencyResolver(IoCService.Instance.Container);

            Startup += DoStartup;
            Exit += DoExit;
            DispatcherUnhandledException += DoException;
        }

        private void DoStartup(object sender, StartupEventArgs e)
        {
            Log.Information("Application Started");
        }

        private void DoException(object sender, DispatcherUnhandledExceptionEventArgs e)
        {
            Log.Fatal(e.Exception, "Unhandled dispatcher exception");
        }

        private void DoExit(object sender, ExitEventArgs e)
        {
            IoCService.Instance.Dispose();
            Log.Information("Application Exit");
        }
    }
}