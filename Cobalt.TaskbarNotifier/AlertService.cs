using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Reactive.Linq;
using Cobalt.Common.Analysis;
using Cobalt.Common.Analysis.OutputTypes;
using Cobalt.Common.Data;
using Cobalt.Common.IoC;
using Cobalt.Common.Transmission.Messages;
using Cobalt.Common.Util;
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

        private Dictionary<long, IDisposable> AlertWatchers { get; set; }

        public void StartMonitoring()
        {
            Resources?.Dispose();
            Resources = GlobalResources.Subscope();
            AlertWatchers = new Dictionary<long, IDisposable>();

            GetAppDurationForDay(
                    new Appl {Id = 6, Path = @"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe"},
                    DateTime.Today, DateTime.Now.AddSeconds(15))
                .Subscribe(x => Debug.WriteLine($"{DateTime.Now}: chrome at {x}"));

            Entities.GetAlertChanges().Subscribe(c =>
            {
                switch (c.ChangeType)
                {
                    case ChangeType.Remove:
                        DemonitorAlert(c.AssociatedEntity);
                        break;
                    case ChangeType.Add:
                        MonitorAlert(c.AssociatedEntity);
                        break;
                    case ChangeType.Modify:
                        DemonitorAlert(c.AssociatedEntity);
                        MonitorAlert(c.AssociatedEntity);
                        break;
                }
            });
        }

        public void MonitorAlert(Alert alert)
        {
            var durations = GetEffectiveTimeRange(DateTime.Today, alert.Range)
                .Select(x =>
                    alert is AppAlert appAlert
                        ? GetAppDurationForDay(appAlert.App, x.Item1, x.Item2.Value)
                        : GetAppDurationForDay(null, x.Item1, x.Item2.Value));
            AlertWatchers[alert.Id] = durations.CombineLatest(x => TimeSpan.FromTicks(x.Sum(ti => ti.Ticks)))
                .Subscribe(dur =>
                {
                    if (dur >= alert.MaxDuration)
                    {
                        //do stuff
                    }
                    else if (dur >= alert.MaxDuration - alert.ReminderOffset)
                    {
                        //send message
                        //shit run this once only
                    }
                }).ManagedBy(Resources);
        }

        public void DemonitorAlert(Alert alert)
        {
            AlertWatchers[alert.Id].Dispose();
        }

        public IEnumerable<(DateTime, DateTime?)> GetEffectiveTimeRange(DateTime today, AlertRange range)
        {
            if (range is OnceAlertRange once && once.StartTimestamp.Date == today)
            {
                yield return (once.StartTimestamp, once.EndTimestamp);
            }
            else if (range is RepeatingAlertRange repeat)
            {
                var start = DateTime.MinValue;
                var end = DateTime.MaxValue;
                switch (repeat.RepeatType)
                {
                    case RepeatType.Daily:
                        start = today;
                        end = today.AddDays(1);
                        break;
                    case RepeatType.Weekly:
                        start = today.StartOfWeek();
                        end = today.EndOfWeek();
                        break;
                    case RepeatType.Weekday:
                        start = today.StartOfWeek().AddDays(1);
                        end = today.EndOfWeek().AddDays(-1);
                        break;
                    case RepeatType.Weekend:
                        start = today.DayOfWeek == DayOfWeek.Sunday ? today.AddDays(-1) : today.EndOfWeek().AddDays(-1);
                        end = start.AddDays(2);
                        break;
                    case RepeatType.Monthly:
                        start = today.StartOfMonth();
                        end = today.EndOfMonth();
                        break;
                }

                for (var current = start; current <= today && current < end; current = current.AddDays(1))
                    yield return AdjustedTimeRange(current, repeat.DailyStartOffset, repeat.DailyEndOffset);
            }
        }

        public (DateTime, DateTime) AdjustedTimeRange(DateTime day, TimeSpan start, TimeSpan end)
        {
            return (day + start, day.AddDays(1) - end);
        }

        public IObservable<TimeSpan> GetAppDurationForDay(Appl app, DateTime start, DateTime end)
        {
            if (start.Date != DateTime.Today)
                return Statistics.GetAppDuration(app, start, end)
                    .Select(x => x.Value);

            var tick = TimeSpan.FromSeconds(1);
            var timer = Observable.Timer(tick, tick).TakeWhile(_ => DateTime.Now <= end);

            var appDurs = Statistics.GetAppDuration(app, start, end, true)
                .Scan(new Usage<TimeSpan>(),
                    (a1, a2) => new Usage<TimeSpan>(a1.Value + a2.Value, a2.JustStarted));


            var prevAppDurs = appDurs.Where(x => !x.JustStarted);
            var justStartedAppDurs = prevAppDurs.Select(x => Observable.Return(x)).Merge(appDurs
                .Where(x => x.JustStarted)
                .Select(x => timer.Select(y =>
                    new Usage<TimeSpan>(TimeSpan.FromSeconds(y) + x.Value, x.JustStarted)).StartWith(x))
            ).Switch();

            return justStartedAppDurs
                .Select(x => x.Value);
        }
    }
}