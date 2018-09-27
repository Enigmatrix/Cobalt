using System;
using System.Collections.Generic;
using System.Reactive.Linq;
using Cobalt.Common.Analysis;
using Cobalt.Common.Analysis.OutputTypes;
using Cobalt.Common.Data;
using Cobalt.Common.IoC;
using Cobalt.Common.UI.Util;
using Cobalt.Common.UI.ViewModels;
using Cobalt.Common.Util;

namespace Cobalt.ViewModels.Pages
{
    public class HomePageViewModel : PageViewModel
    {

        public HomePageViewModel(IResourceScope scope) : base(scope)
        {
        }

        public IObservable<AppDurationViewModel> AppDurations { get; set; }

        public IObservable<AppDurationViewModel> WeekAppDurations { get; set; }

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

        public TimeSpan HoursSpentDay { get; set; }

        public TimeSpan HoursSpentWeek { get; set; }

        public IObservable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> HourlyChunks { get;set; }

        public IObservable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> DayChunks { get; set; }

        public IObservable<AppUsageViewModel> AppUsagesToday { get; set; }

        public IObservable<TagDurationViewModel> TagDurations { get; set; }

        protected override void OnActivate(IResourceScope res)
        {
            var stats = res.Resolve<IAppStatsStreamService>();

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

            HourlyChunks =
                stats.GetChunkedAppDurations(TimeSpan.FromHours(1), d => d.Date.AddHours(d.Hour), DateTime.Today);

            DayChunks =
                stats.GetChunkedAppDurations(TimeSpan.FromDays(1), d => d.Date, DateTime.Today.StartOfWeek(), DateTime.Today.EndOfWeek());

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

        protected override void OnDeactivate(bool close, IResourceScope res)
        {
            AppDurations = null;
        }
    }
}