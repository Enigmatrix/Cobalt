using System;
using System.Reactive.Linq;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Repository;
using Cobalt.Common.IoC;
using Cobalt.Common.Transmission;
using Cobalt.Common.Transmission.Messages;
using Cobalt.Engine.Util;
using Serilog;

namespace Cobalt.Engine
{
    public class Program
    {
        public static void Main(string[] args)
        {
            try
            {
                Run();
            }
            catch (Exception e)
            {
                Log.Information($"Error, exception raised: {e}");
            }
        }

        public static void Run()
        {
            Log.Logger = new LoggerConfiguration()
                .WriteTo.File("./log.txt")
                .CreateLogger();
            Log.Information("NEW SESSION");

            var repository = IoCService.Instance.Resolve<IDbRepository>();
            var transmitter = IoCService.Instance.Resolve<ITransmissionServer>();
            var hookMgr = new HookManager();
            var appWatcher = new AppWatcher(hookMgr);
            //var idleWatcher = new InteractionWatcher(hookMgr);
            var sysWatcher = new SystemWatcher(MessageWindow.Instance);

            appWatcher.ForegroundAppUsageObtained += (_, e) =>
            {
                try
                {
                    var prevAppUsage = e.PreviousAppUsage;
                    var newApp = e.NewApp;

                    //check if the app path is already stored in the database
                    var appId = repository.FindAppIdByPath(prevAppUsage.App.Path);
                    if (appId == null)
                        //if not, add a new app with that path
                        repository.AddApp(prevAppUsage.App);
                    else
                        //else use the previously existing app's identity
                        prevAppUsage.App.Id = appId.Value;

                    //store the incoming app's path too - leads to the Exists check of the previous statements to be redundant
                    //but hey i prefer consistency over perf sometimes
                    if (newApp != null)
                    {
                        var newappId = repository.FindAppIdByPath(newApp.Path);
                        if (newappId == null)
                            repository.AddApp(newApp);
                        else
                            newApp.Id = newappId.Value;
                    }

                    //broadcast foreground app switch to all clients
                    transmitter.Send(new AppSwitchMessage(prevAppUsage, newApp));
                    //then store the app usage
                    repository.AddAppUsage(prevAppUsage);

                    //LogAppSwitch(prevAppUsage, newApp);
                }
                catch (Exception ex)
                {
                    Log.Information($"Error, exception raised inside AppWatcher: {ex}");
                }
            };


            /*idleWatcher.IdleObtained += (_, e) =>
            {
                repository.AddInteraction(e.Interaction);
            };*/

            var systemStateChanges = Observable.FromEventPattern<SystemStateChangedArgs>(
                    handler => sysWatcher.SystemMainStateChanged += handler,
                    handler => sysWatcher.SystemMainStateChanged -= handler)
                .Select(x => x.EventArgs.ChangedToState).Publish();

            var notMonitorOff = systemStateChanges.Where(x => x != SystemStateChange.MonitorOff);
            //monitor off usually comes before sleep/hibernate by a few milliseconds, so we are throttling it!
            var monitorOff = systemStateChanges.Where(x => x == SystemStateChange.MonitorOff)
                .Throttle(TimeSpan.FromMilliseconds(1000));
            notMonitorOff.Merge(monitorOff).Subscribe(x =>
            {
                Log.Information($"STATE CHANGE TO: {x}");
                if (x.IsStartRecordingEvent())
                    appWatcher.StartRecordingWith(x.ToStartReason());
                else appWatcher.EndRecordingWith(x.ToEndReason());
            });
            systemStateChanges.Connect();

            appWatcher.EventLoop();
        }

        private static void LogAppSwitch(AppUsage prevAppUsage, App newApp)
        {
            Log.Information("[{start} - {end} ({duration})] {{{startReason}, {endReason}}} {prev} -> {new}",
                prevAppUsage.StartTimestamp.ToString("HH:mm:ss.fff tt"),
                prevAppUsage.EndTimestamp.ToString("HH:mm:ss.fff tt"),
                prevAppUsage.EndTimestamp - prevAppUsage.StartTimestamp,
                prevAppUsage.UsageStartReason,
                prevAppUsage.UsageEndReason,
                prevAppUsage.App.Path,
                newApp?.Path);
        }
    }
}