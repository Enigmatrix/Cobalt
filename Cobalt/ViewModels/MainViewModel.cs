using System;
using System.Linq;
using System.Reactive.Linq;
using Caliburn.Micro;
using Cobalt.Common.Analysis;
using Cobalt.Common.Data;
using Cobalt.Common.IoC;
using Cobalt.Common.UI;
using Cobalt.Common.UI.Util;
using Cobalt.Common.UI.ViewModels;

namespace Cobalt.ViewModels
{
    public interface IMainViewModel : IViewModel
    {
        NextLevelBindableCollection<IAppUsageViewModel> AppUsages { get; }
        IAppUsageViewModel CurrentAppUsage { get; set; }
        long AppUsageCount { get; set; }
    }

    public class MainViewModel : ViewModelBase, IMainViewModel
    {
        private long _appUsageCount;
        private IAppUsageViewModel _currentAppUsage;

        public MainViewModel(IResourceScope scope, IAppStatsStreamService stats)
        {
            Stats = stats;
            Resources = scope;
            AppUsages = new NextLevelBindableCollection<IAppUsageViewModel>();
        }

        private IResourceScope Resources { get; }

        private IAppStatsStreamService Stats { get; }

        //view should display all AppUsages and CurrentAppUsage

        public NextLevelBindableCollection<IAppUsageViewModel> AppUsages { get; }

        public IAppUsageViewModel CurrentAppUsage
        {
            get => _currentAppUsage;
            set => Set(ref _currentAppUsage, value);
        }

        public long AppUsageCount
        {
            get => _appUsageCount;
            set => Set(ref _appUsageCount, value);
        }

        protected override void OnActivate()
        {
            var appUsagesStream = Stats.GetAppUsages(DateTime.Today.AddDays(-2), DateTime.Today.AddDays(-1)).Publish();
            var appUsageIncremetor = Resources.Resolve<IDurationIncrementor>();

            var lastAppUsageTime = DateTime.Now;
            appUsagesStream
                .Where(x => !x.JustStarted)
                .Select(x => (IAppUsageViewModel)new AppUsageViewModel(x.Value))
                .Buffer(TimeSpan.FromMilliseconds(20))
                .Subscribe(x =>
                {
                    AppUsages.AddRange(x);
                });
            /*
            appUsagesStream.Where(x => !x.JustStarted)
                .LongCount()
                .ObserveOnDispatcher()
                .Subscribe(x => AppUsageCount++);

            appUsagesStream.Where(x => x.JustStarted)
                .Subscribe(x =>
                {
                    if (CurrentAppUsage == null) AppUsageCount++;

                    appUsageIncremetor.Release();
                    CurrentAppUsage = new AppUsageViewModel(new AppUsage
                    {
                        App = x.Value.App,
                        StartTimestamp = lastAppUsageTime,
                        EndTimestamp = lastAppUsageTime
                    });
                    appUsageIncremetor.Increment(CurrentAppUsage);
                });*/


            appUsagesStream.Connect().ManageUsing(Resources);
        }

        protected override void OnDeactivate(bool close)
        {
            Resources.Dispose();
        }
    }
}