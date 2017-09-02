﻿using System.Windows.Threading;
using Cobalt.Common.UI;
using Serilog;

namespace Cobalt.TaskbarNotifier
{
    /// <inheritdoc />
    /// <summary>
    ///     Interaction logic for App.xaml
    /// </summary>
    public partial class App
    {
    }

    public class AppBoostrapper : Bootstrapper<IMainViewModel>
    {
        protected override void PrepareApplication()
        {
            Log.Logger = new LoggerConfiguration()
                .WriteTo.File("./tn-log.txt")
                .CreateLogger();
            Log.Information("NEW SESSION");
        }

        protected override void OnUnhandledException(object sender, DispatcherUnhandledExceptionEventArgs e)
        {
            Log.Information($"Exception raised in TaskbarNotifier: {e}");
        }
    }
}