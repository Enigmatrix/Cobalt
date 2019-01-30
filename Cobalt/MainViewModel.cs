using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Reactive.Linq;
using System.Text;
using System.Threading.Tasks;
using Cobalt.Common.Data.Entities;
using DynamicData;
using ReactiveUI;

namespace Cobalt
{
    public class MainViewModel : ReactiveObject
    {
        private readonly ReadOnlyObservableCollection<AppUsageViewModel> _appUsages;

        public ReadOnlyObservableCollection<AppUsageViewModel> AppUsages => _appUsages;

        public MainViewModel(Service svc)
        {
            svc.Switches().Select(x => new AppUsageViewModel(x.Previous))
                .ToObservableChangeSet()
                .ObserveOnDispatcher()
                .Bind(out _appUsages)
                .DisposeMany()
                .Subscribe();

        }
    }
}
