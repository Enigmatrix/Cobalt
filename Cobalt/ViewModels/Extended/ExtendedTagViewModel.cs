using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Collections.Specialized;
using System.Reactive.Linq;
using Cobalt.Common.Analysis;
using Cobalt.Common.Analysis.OutputTypes;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Repository;
using Cobalt.Common.IoC;
using Cobalt.Common.UI.Util;
using Cobalt.Common.UI.ViewModels;

namespace Cobalt.ViewModels.Extended
{
    public class ExtendedTagViewModel : TagViewModel
    {
        private IObservable<AppDurationViewModel> _taggedAppDurationsToday;
        private ObservableCollection<AppViewModel> _taggedApps;
        private IObservable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> _taggedAppsHourlyChunks;

        public ExtendedTagViewModel(Tag tag, IResourceScope res, IAppStatsStreamService stats, IDbRepository repo) :
            base(tag)
        {
            Statistics = stats;
            Repository = repo;
            Resources = res;
        }

        public IResourceScope Resources { get; set; }

        public IDbRepository Repository { get; set; }

        public IAppStatsStreamService Statistics { get; set; }

        public ObservableCollection<AppViewModel> TaggedApps
        {
            get
            {
                if (_taggedApps != null) return _taggedApps;

                TaggedApps = new ObservableCollection<AppViewModel>();
                Repository.GetAppsWithTag((Tag) Entity).Subscribe(x => TaggedApps.Add(new AppViewModel(x)));
                return _taggedApps;
            }
            set
            {
                Set(ref _taggedApps, value);
                _taggedApps.CollectionChanged += SetStatistics;
            }
        }

        public IObservable<AppDurationViewModel> TaggedAppDurationsToday
        {
            get => _taggedAppDurationsToday;
            set => Set(ref _taggedAppDurationsToday, value);
        }

        private void SetStatistics(object sender, NotifyCollectionChangedEventArgs e)
        {
            var appIncrementor = Resources.Resolve<IDurationIncrementor>();
            TaggedAppDurationsToday = Statistics.GetTaggedAppDurations((Tag) Entity, DateTime.Today)
                .Select(x =>
                {
                    var appDur = new AppDurationViewModel(x.App);

                    x.Duration
                        .Subscribe(d => { appDur.DurationIncrement(d, appIncrementor); })
                        .ManageUsing(Resources);

                    return appDur;
                });

            var appUsagesStream = Statistics.GetAppUsages((Tag)Entity, DateTime.Today, DateTime.Now);

            TaggedAppsHourlyChunks = appUsagesStream
                .SelectMany(u => SplitUsageIntoChunks(u, TimeSpan.FromHours(1), d => d.Date.AddHours(d.Hour)));
        }

        public IObservable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> TaggedAppsHourlyChunks
        {
            get => _taggedAppsHourlyChunks;
            set => Set(ref _taggedAppsHourlyChunks, value);
        }


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
    }
}