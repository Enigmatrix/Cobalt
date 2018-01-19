﻿using System;
using System.Collections.Generic;
using System.Reactive.Linq;
using Caliburn.Micro;
using Cobalt.Common.Analysis;
using Cobalt.Common.Analysis.OutputTypes;
using Cobalt.Common.Data;
using Cobalt.Common.IoC;
using Cobalt.Common.UI;
using Cobalt.Common.UI.Util;
using Cobalt.Common.UI.ViewModels;
using Cobalt.Common.Util;
using LiveCharts;

namespace Cobalt.ViewModels
{
    public interface IMainViewModel : IViewModel
    {
        IObservable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> HourlyChunks { get; set; }
    }

    public class SecretViewModel : ViewModelBase, IMainViewModel
    {
        private IObservable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> _hourlyChunks;
        private BindableCollection<IAppDurationViewModel> _appDurations = new BindableCollection<IAppDurationViewModel>();

        public SecretViewModel(IResourceScope scope, IAppStatsStreamService stats)
        {
            Resources = scope;
            Stats = stats;
        }

        public BindableCollection<IAppDurationViewModel> AppDurations
        {
            get => _appDurations;
            set => Set(ref _appDurations, value);
        }
        public Func<double, string> HourFormatter => x => x / 600000000 + "min";
        public Func<double, string> DayHourFormatter => x => (x % 12 == 0? 12: x%12) + (x >= 12 ? "pm" : "am");

        private IResourceScope Resources { get; }

        private IAppStatsStreamService Stats { get; }

        private static IEqualityComparer<App> PathEquality { get; }
            = new SelectorEqualityComparer<App, string>(a => a.Path);

        public IObservable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> HourlyChunks
        {
            get => _hourlyChunks;
            set => Set(ref _hourlyChunks, value);
        }

        protected override void OnActivate()
        {
            var appUsagesStream = Stats.GetAppUsages(DateTime.Today, DateTime.Now);//.Publish();
            var appDurationsStream = Stats.GetAppDurations(DateTime.Today);//.Publish();
            var appIncrementor = Resources.Resolve<IDurationIncrementor>();

            HourlyChunks = appUsagesStream
                .SelectMany(SplitUsageIntoHourChunks);


            appDurationsStream
                .Select(x =>
                {
                    var appDur = new AppDurationViewModel(x.App);

                    x.Duration
                        .Subscribe(d => { appDur.DurationIncrement(d, appIncrementor); })
                        .ManageUsing(Resources);

                    return appDur;
                })
                .ObserveOnDispatcher()
                .Subscribe(x =>
                    AppDurations.Add(x));

            //appUsagesStream.Connect().ManageUsing(Resources);
        }

        //TODO move this to common

        private IEnumerable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> SplitUsageIntoHourChunks(Usage<AppUsage> usage)
        {
            var appUsage = usage.Value;
            var start = appUsage.StartTimestamp;
            var end = appUsage.EndTimestamp;
            while (start < end)
            {
                var startHr = start.Subtract(new TimeSpan(0, 0, start.Minute, start.Second, start.Millisecond));
                var endHr = Min(startHr.AddHours(1), end);
                if(!(endHr < end) && usage.JustStarted)
                    yield return new Usage<(App,DateTime,TimeSpan)>(justStarted:true, value: (appUsage.App, startHr, TimeSpan.Zero));
                else
                    yield return new Usage<(App,DateTime,TimeSpan)>((appUsage.App, startHr, endHr - start));    
                start = endHr;
            }
        }

        public T Min<T>(T a, T b) where T : IComparable<T>
        {
            return a.CompareTo(b) < 0 ? a : b;
        }

        protected override void OnDeactivate(bool close)
        {
            Resources.Dispose();
        }
    }
}