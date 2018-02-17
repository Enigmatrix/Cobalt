using Cobalt.Common.Analysis;
using Cobalt.Common.IoC;
using Cobalt.Common.Util;
using Cobalt.Common.Analysis.OutputTypes;
using System;
using System.Linq;
using System.Reactive.Linq;
using System.Diagnostics;
using Appl = Cobalt.Common.Data.App;

namespace Cobalt.TaskbarNotifier
{
    public class AlertService
    {
        public AlertService(IResourceScope res, IEntityStreamService entities, IAppStatsStreamService stats)
        {
            Entities = entities;
            Statistics = stats;
            GlobalResources = res;
            MidnightNotifier.DayChanged().Subscribe(x => StartMonitoring());
        }

        public IEntityStreamService Entities { get; }
        public IAppStatsStreamService Statistics { get; }
        public IResourceScope GlobalResources { get; }
        public IResourceScope Resources { get; set; }

        public void StartMonitoring()
        {
            Resources?.Dispose();
            Resources = GlobalResources.Subscope();
            /*
            var alertWatchers = new Dictionary<int, IDisposable>();

            Entities.GetAlertChanges().Subscribe(c => {
                switch (c.ChangeType)
                {
                    case ChangeType.Remove:
                        alertWatchers[c.AssociatedAlert.Id].Dispose();
                        break;
                    case ChangeType.Add:
                        alertWatchers[c.AssociatedAlert.Id] = MonitorAlert(c.AssociatedAlert);
                        break;
                    case ChangeType.Modify:
                        alertWatchers[c.AssociatedAlert.Id].Dispose();
                        alertWatchers[c.AssociatedAlert.Id] = MonitorAlert(c.AssociatedAlert);
                        break;
                }
            });

        }

        public IDisposable MonitorAlert(Alert alert)
        {
            return null;
            */
        }

        public IObservable<TimeSpan> GetAppDuration(Appl app, DateTime? start = null, DateTime? end = null)
        {
            var tick = TimeSpan.FromSeconds(1);
            var timer = Observable.Timer(tick, tick);

            var appDurs = Statistics.GetAppDurations(DateTime.MinValue)
                //TODO make a custom one -_- instead of filtering
                .Where(a => a.App.Path == app.Path)
                .SelectMany(x => x.Duration.Scan(new Usage<TimeSpan>(),
                    (a1, a2) => new Usage<TimeSpan>((a1.Value + a2.Value), a2.JustStarted)));

            var prevAppDurs = appDurs.Where(x => !x.JustStarted);

            var justStartedAppDurs = appDurs.Where(x => x.JustStarted)
                //make a new timer everytime a app is on foreground
                .Select(x => timer.Select(y =>
                    new Usage<TimeSpan>(TimeSpan.FromSeconds(y) + x.Value, x.JustStarted)).StartWith(x))
                .Switch();

            return prevAppDurs.Merge(justStartedAppDurs).Select(x => x.Value);
        }

    }
}
