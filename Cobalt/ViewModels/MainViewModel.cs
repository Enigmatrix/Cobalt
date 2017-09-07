using System;
using System.Net;
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
        BindableCollection<IAppUsageViewModel> AppUsages { get; }
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
            AppUsages = new BindableCollection<IAppUsageViewModel>();
        }

        private IResourceScope Resources { get; }

        private IAppStatsStreamService Stats { get; }

        //view should display all AppUsages and CurrentAppUsage

        public BindableCollection<IAppUsageViewModel> AppUsages { get; }

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
            var appUsages = Stats.GetAppUsages(DateTime.Today);
            var appUsageIncremetor = Resources.Resolve<IDurationIncrementor>();

            var lastAppUsageTime = DateTime.Now;
            appUsages
                .Subscribe(x =>
                {
                    if (x.JustStarted)
                    {
                        //first time for new app
                        if (CurrentAppUsage == null) AppUsageCount++;

                        appUsageIncremetor.Release();
                        CurrentAppUsage = new AppUsageViewModel(new AppUsage
                        {
                            App = x.Value.App,
                            StartTimestamp = lastAppUsageTime,
                            EndTimestamp = lastAppUsageTime
                        });
                        appUsageIncremetor.Increment(CurrentAppUsage);

                    }
                    else
                    {
                        lastAppUsageTime = x.Value.EndTimestamp;
                        AppUsages.Add(new AppUsageViewModel(x.Value));

                        AppUsageCount++;
                    }
                }).ManageUsing(Resources);
        }
    }
}