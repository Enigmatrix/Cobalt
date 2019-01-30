using System;
using System.Collections.ObjectModel;
using System.Reactive.Linq;
using DynamicData;
using ReactiveUI;

namespace Cobalt
{
    public class MainViewModel : ReactiveObject
    {
        private readonly ReadOnlyObservableCollection<AppUsageViewModel> _appUsages;

        public MainViewModel(Service svc)
        {
            svc.Switches().Select(x => new AppUsageViewModel(x.Previous))
                .ToObservableChangeSet()
                .ObserveOnDispatcher()
                .Bind(out _appUsages)
                .DisposeMany()
                .Subscribe();
        }

        public ReadOnlyObservableCollection<AppUsageViewModel> AppUsages => _appUsages;
    }
}