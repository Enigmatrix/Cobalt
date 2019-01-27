using System;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.Data.Repositories;
using Cobalt.Common.IoC;
using Cobalt.Common.Transmission;
using Cobalt.Common.Transmission.Messages;
using Serilog;

namespace Cobalt.Engine
{
    public class Program
    {
        public static void Main(string[] args)
        {
            var resolver = IoCService.Instance.Resolve<AppInfoResolver>();
            var watcher = IoCService.Instance.Resolve<AppWatcher>();
            var server = IoCService.Instance.Resolve<ITransmissionServer>();
            var repo = IoCService.Instance.Resolve<IDbRepository>();

            App InsertWithDetailsOrRetreive(App app)
            {
                if (repo.AppIdByPath(app)) return app;
                app = resolver.WithDetails(app);
                repo.Insert(app);
                return app;
            }

            Log.Information("Started");

            watcher.Switches.Subscribe(x =>
            {
                x.Previous.App = InsertWithDetailsOrRetreive(x.Previous.App);

                repo.Insert(x.Previous);

                x.Active = InsertWithDetailsOrRetreive(x.Active);

                server.Send(new AppSwitchMessage
                {
                    AppUsageId = x.Previous.Id,
                    ActiveAppId = x.Active.Id
                });
            });

            watcher.Start();
            EventLoop();
        }

        private static void EventLoop()
        {
            //keep getting messages
            while (true)
            {
                if (Win32.GetMessage(out var msg, IntPtr.Zero, 0, 0) == 0) break;

                //even ms docs say that you dont need to understand these
                Win32.TranslateMessage(ref msg);
                Win32.DispatchMessage(ref msg);
            }
        }
    }
}