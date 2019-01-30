using System;
using System.Reactive.Disposables;
using System.Reactive.Linq;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.IoC;
using Cobalt.Common.Services;
using DynamicData;
using Serilog;

namespace Cobalt.Alerts
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
                Log.Fatal(e, "General Error");
            }
        }

        private static void Run()
        {
            var entitySvc = IoCService.Instance.Resolve<IEntityService>();
            var statsSvc = IoCService.Instance.Resolve<IStatisticsService>();

            IObservable<TimeSpan> GetDuration(Alert a, DateTime? start, DateTime? end)
            {
                switch (a)
                {
                    case AppAlert aa:
                        return statsSvc.GetAppDuration(aa.App, start, end);
                    case TagAlert ta:
                        return statsSvc.GetTagDuration(ta.Tag, start, end);
                }
                throw new Exception("Alert neither TagAlert nor AppAlert");
            }

            entitySvc.GetAlerts()
                .Filter(x => x.Enabled)
                .SubscribeMany(alert =>
                {
                    var (start, end) = GetStartEnd(alert.TimeRange);
                    var reminders = entitySvc.GetRemindersForAlert(alert);
                    var durations = GetDuration(alert, start, end).Publish();

                    durations.FirstAsync(dur => dur >= alert.MaxDuration)
                        .Subscribe(_ => ActOnAlert(alert));

                    var reminderWatchers = reminders.SubscribeMany(r =>
                        {
                            var reminderDur = alert.MaxDuration - r.Offset;
                            return durations.FirstAsync(dur => dur >= reminderDur)
                                .Subscribe(_ => ActOnReminder(alert, r));
                        })
                        .DisposeMany()
                        .Subscribe();

                    return new CompositeDisposable(durations.Connect(), reminderWatchers);
                })
                .DisposeMany()
                .Subscribe();

            while (true)
            {
            }
        }

        public static (DateTime? Start, DateTime? End) GetStartEnd(TimeRange time)
        {
            switch (time)
            {
                case OnceTimeRange once:
                    return (once.Start, once.End);
                case RepeatingTimeRange repeat:
                    throw new NotImplementedException();
            }
            throw new Exception("TimeRange neither OnceTimeRange nor RepeatingTimeRange");
        }

        private static void ActOnAlert(Alert alert)
        {
            Log.Information("Firing RunAction for {@alert}", alert);
            try
            {
                //TODO
                switch (alert.Action)
                {
                    case MessageRunAction ra:
                        
                        break;
                    case CustomMessageRunAction ra:
                        break;
                    case KillRunAction ra:
                        break;
                    case ScriptMessageRunAction ra:
                        break;
                }
            }
            catch (Exception e)
            {
                Log.Fatal(e, "Error when running RunAction");
            }
        }

        private static void ActOnReminder(Alert alert, Reminder reminder)
        {
            Log.Information("Firing ReminderAction for {@reminder}", reminder);
            try
            {
                //TODO
                switch (reminder.Action)
                {
                    case CustomWarnReminderAction ra:
                        break;
                    case ScriptReminderAction ra:
                        break;
                    case WarnReminderAction ra:
                        break;
                }
            }
            catch (Exception e)
            {
                Log.Fatal(e, "Error when running ReminderAction");
            }
        }
    }
}