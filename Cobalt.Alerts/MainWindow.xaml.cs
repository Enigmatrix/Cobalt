using System;
using System.Diagnostics;
using System.Reactive.Disposables;
using System.Reactive.Linq;
using System.Windows;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.IoC;
using Cobalt.Common.Services;
using Cobalt.Common.Util;
using DynamicData;
using Serilog;
using ToastNotifications;
using ToastNotifications.Core;
using ToastNotifications.Lifetime;
using ToastNotifications.Messages;
using ToastNotifications.Position;

namespace Cobalt.Alerts
{
    /// <summary>
    ///     Interaction logic for MainWindow.xaml
    /// </summary>
    public partial class MainWindow
    {
        public MainWindow()
        {
            InitializeComponent();
        }

        private static readonly Notifier Notifier = new Notifier(cfg =>
        {
            cfg.PositionProvider = new PrimaryScreenPositionProvider(Corner.BottomRight, 5, 5);
            cfg.LifetimeSupervisor = new TimeAndCountBasedLifetimeSupervisor(
                TimeSpan.FromSeconds(5),
                MaximumNotificationCount.UnlimitedNotifications());
            cfg.Dispatcher = Application.Current.Dispatcher;
        });

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
                return Throw.Unreachable<IObservable<TimeSpan>>();
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
                .DisposeMany();
        }

        private static (DateTime? Start, DateTime? End) GetStartEnd(TimeRange time)
        {
            switch (time)
            {
                case OnceTimeRange once:
                    return (once.Start, once.End);
                case RepeatingTimeRange repeat:
                    switch (repeat.Type)
                    {
                        case RepeatingTimeRangeType.Daily:
                            return (DateTime.Today, DateTime.Today.AddDays(1));
                        case RepeatingTimeRangeType.Weekly:
                            return (DateTime.Today.StartOfWeek(), DateTime.Today.EndOfWeek());
                        case RepeatingTimeRangeType.Monthly:
                            return (DateTime.Today.StartOfMonth(), DateTime.Today.EndOfMonth());
                    }
                    break;
            }
            return Throw.Unreachable<(DateTime, DateTime)>();
        }

        private static void ActOnAlert(Alert alert)
        {
            Log.Information("Firing RunAction for {@alert}", alert);
            try
            {
                switch (alert.Action)
                {
                    case MessageRunAction _:
                        var entityMessage = "";
                        switch (alert)
                        {
                            case AppAlert a:
                                entityMessage = $"App {a.App.Name}";
                                break;
                            case TagAlert a:
                                entityMessage = $"Tag {a.Tag.Name}";
                                break;
                        }
                        Notifier.ShowError($"Time is up for {entityMessage}!");
                        break;
                    case CustomMessageRunAction ra:
                        Notifier.ShowError(ra.Message);
                        break;
                    case KillRunAction _:
                        //TODO make a thread for keep on killing process of that path
                        break;
                    case ScriptMessageRunAction ra:
                        RunScript(ra.Script);
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
                switch (reminder.Action)
                {
                    case CustomWarnReminderAction ra:
                        Notifier.ShowInformation(ra.Warning);
                        break;
                    case ScriptReminderAction ra:
                        RunScript(ra.Script);
                        break;
                    case WarnReminderAction _:
                        var entityMessage = "";
                        switch (alert)
                        {
                            case AppAlert a:
                                entityMessage = a.App.Name;
                                break;
                            case TagAlert a:
                                entityMessage = a.Tag.Name;
                                break;
                        }
                        Notifier.ShowInformation($"Reminder for {entityMessage} ({reminder.Offset})");
                        break;
                }
            }
            catch (Exception e)
            {
                Log.Fatal(e, "Error when running ReminderAction");
            }
        }

        private static void RunScript(string script)
        {
            try
            {
                var startInfo = new ProcessStartInfo
                {
                    Arguments = script,
                    UseShellExecute = true
                };
                var proc = Process.Start(startInfo);
                proc.Start();
            }
            catch (Exception e)
            {
                Log.Fatal(e, "Error when running Script {script}", script);
            }
        }

        private void OnInit(object sender, EventArgs _)
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
    }
}