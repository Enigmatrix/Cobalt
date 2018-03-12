using System;
using System.Collections.Generic;
using System.Reactive.Linq;
using Cobalt.Common.Analysis;
using Cobalt.Common.Analysis.OutputTypes;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Repository;
using Cobalt.Common.IoC;
using Cobalt.Common.UI.Util;
using Cobalt.Common.UI.ViewModels;
using Cobalt.Common.Util;

namespace Cobalt.ViewModels.Pages
{
    public class HomePageViewModel : PageViewModel
    {
        private IObservable<AppDurationViewModel> _appDurations;
        private IObservable<AppUsageViewModel> _appUsagesToday;
        private IObservable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> _dayChunks;
        private IObservable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> _hourlyChunks;
        private TimeSpan _hoursSpentDay;
        private TimeSpan _hoursSpentWeek;
        private IObservable<TagDurationViewModel> _tagDurations;
        private IObservable<AppDurationViewModel> _weekAppDurations;

        public HomePageViewModel(IResourceScope scope) : base(scope)
        {
        }

        public IObservable<AppDurationViewModel> AppDurations
        {
            get => _appDurations;
            set => Set(ref _appDurations, value);
        }

        public IObservable<AppDurationViewModel> WeekAppDurations
        {
            get => _weekAppDurations;
            set => Set(ref _weekAppDurations, value);
        }

        public Func<double, string> HourFormatter => x => x / 600000000 + "min";
        public Func<double, string> DayFormatter => x => x == 0 ? "" : x / 36000000000 + "h";
        public Func<double, string> DayHourFormatter => x => (x % 12 == 0 ? 12 : x % 12) + (x >= 12 ? "p" : "a");
        public Func<double, string> DayOfWeekFormatter => x => ((DayOfWeek) (int) x).ToString();

        public static DateTime WeekStart => DateTime.Today.StartOfWeek();
        public static DateTime WeekEnd => DateTime.Today.EndOfWeek();
        public static TimeSpan DayDuration => TimeSpan.FromDays(1);

        public static DateTime DayStart => DateTime.Today;
        public static DateTime DayEnd => DateTime.Today.AddDays(1);
        public static TimeSpan HourDuration => TimeSpan.FromHours(1);

        public TimeSpan HoursSpentDay
        {
            get => _hoursSpentDay;
            set => Set(ref _hoursSpentDay, value);
        }

        public TimeSpan HoursSpentWeek
        {
            get => _hoursSpentWeek;
            set => Set(ref _hoursSpentWeek, value);
        }

        public IObservable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> HourlyChunks
        {
            get => _hourlyChunks;
            set => Set(ref _hourlyChunks, value);
        }

        public IObservable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> DayChunks
        {
            get => _dayChunks;
            set => Set(ref _dayChunks, value);
        }

        public IObservable<AppUsageViewModel> AppUsagesToday
        {
            get => _appUsagesToday;
            set => Set(ref _appUsagesToday, value);
        }

        public IObservable<TagDurationViewModel> TagDurations
        {
            get => _tagDurations;
            set => Set(ref _tagDurations, value);
        }

        protected override void OnActivate(IResourceScope res)
        {
            var stats = res.Resolve<IAppStatsStreamService>();
            var appUsagesStream = stats.GetAppUsages(DateTime.Today, DateTime.Now); //.Publish();
            var weekAppUsagesStream = stats.GetAppUsages(WeekStart, DateTime.Now);

            var appDurationsStream = stats.GetAppDurations(DateTime.Today);
            var weekAppDurationsStream = stats.GetAppDurations(WeekStart);
            var appIncrementor = res.Resolve<IDurationIncrementor>();
            var weekAppIncrementor = res.Resolve<IDurationIncrementor>();

            stats.GetAppUsageDuration(DateTime.Today)
                .ObserveOnDispatcher()
                .Subscribe(x => HoursSpentDay = x).ManageUsing(Resources);
            stats.GetAppUsageDuration(DateTime.Today.StartOfWeek())
                .ObserveOnDispatcher()
                .Subscribe(x => HoursSpentWeek = x).ManageUsing(Resources);

            AppUsagesToday = stats.GetAppUsages(DateTime.Today).Select(x =>
                /*TODO handle duration animation*/new AppUsageViewModel(x.Value));

            HourlyChunks = appUsagesStream
                .SelectMany(u => SplitUsageIntoChunks(u, TimeSpan.FromHours(1), d => d.Date.AddHours(d.Hour)));

            DayChunks = weekAppUsagesStream
                .SelectMany(u => SplitUsageIntoChunks(u, TimeSpan.FromDays(1), d => d.Date));

            AppDurations = appDurationsStream
                .Select(x =>
                {
                    var appDur = new AppDurationViewModel(x.App);

                    x.Duration
                        .Subscribe(d => { appDur.DurationIncrement(d, appIncrementor); })
                        .ManageUsing(res);

                    return appDur;
                });

            WeekAppDurations = weekAppDurationsStream
                .Select(x =>
                {
                    var appDur = new AppDurationViewModel(x.App);

                    x.Duration
                        .Subscribe(d => { appDur.DurationIncrement(d, weekAppIncrementor); })
                        .ManageUsing(res);

                    return appDur;
                });

            TagDurations = stats.GetTagDurations(DateTime.Today)
                .Select(x =>
                {
                    var appDur = new TagDurationViewModel(x.Tag);

                    x.Duration
                        .Subscribe(d => { appDur.DurationIncrement(d, appIncrementor); })
                        .ManageUsing(res);

                    return appDur;
                });
        }

        //TODO move this to common

        private IEnumerable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> SplitUsageIntoChunks(
            Usage<AppUsage> usage, TimeSpan chunk, Func<DateTime, DateTime> startSelector)
        {
            var appUsage = usage.Value;
            var start = appUsage.StartTimestamp;
            var end = appUsage.EndTimestamp;
            while (start < end)
            {
                var startHr = startSelector(start);
                var endHr = Min(startHr + chunk, end);
                if (!(endHr < end) && usage.JustStarted)
                    yield return new Usage<(App, DateTime, TimeSpan)>(justStarted: true,
                        value: (appUsage.App, startHr, TimeSpan.Zero));
                else
                    yield return new Usage<(App, DateTime, TimeSpan)>((appUsage.App, startHr, endHr - start));
                start = endHr;
            }
        }

        public T Min<T>(T a, T b) where T : IComparable<T>
        {
            return a.CompareTo(b) < 0 ? a : b;
        }

        protected override void OnDeactivate(bool close, IResourceScope res)
        {
            AppDurations = null;
        }
    }
}