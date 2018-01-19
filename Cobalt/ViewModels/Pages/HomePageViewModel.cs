﻿using System;
using System.Collections.Generic;
using System.Linq;
using System.Reactive.Linq;
using System.Text;
using System.Threading.Tasks;
using Caliburn.Micro;
using Cobalt.Common.Analysis;
using Cobalt.Common.IoC;
using Cobalt.Common.UI.ViewModels;

namespace Cobalt.ViewModels.Pages
{
    public class HomePageViewModel : PageViewModel
    {
        private IObservable<IAppDurationViewModel> _appDurations;
        private IObservable<IAppUsageViewModel> _appUsages;

        public HomePageViewModel(IResourceScope scope, IAppStatsStreamService stats)
        {
            Resources = scope;
            Stats = stats;
        }

        public IObservable<IAppDurationViewModel> AppDurations { get => _appDurations; set => Set(ref _appDurations, value); }
        public IObservable<IAppUsageViewModel> AppUsages { get => _appUsages; set => Set(ref _appUsages, value); }


        protected override void OnActivate()
        {
            var appUsagesStream = Stats.GetAppUsages(DateTime.Today);
            var appDurationsStream = Stats.GetAppDurations(DateTime.Today);


        }

        protected override void OnDeactivate(bool close)
        {
            
        }

        public IAppStatsStreamService Stats { get; set; }

        public IResourceScope Resources { get; set; }
    }
}