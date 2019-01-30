using System;
using System.Collections.ObjectModel;
using System.Reactive.Disposables;
using System.Reactive.Linq;
using DynamicData;
using ReactiveUI;

namespace Cobalt
{
    public class MainViewModel : ReactiveObject, ISupportsActivation
    {
        private ReadOnlyObservableCollection<AppUsageViewModel> _appUsages;

        public MainViewModel(Service svc)
        {
            Activator = new ViewModelActivator();
            this.WhenActivated(disposables =>
            {
                svc.Switches().Transform(x => new AppUsageViewModel(x.Previous))
                    .ObserveOnDispatcher()
                    .Bind(out _appUsages)
                    .DisposeMany()
                    .Subscribe()
                    .DisposeWith(disposables);
            });
        }

        public ReadOnlyObservableCollection<AppUsageViewModel> AppUsages => _appUsages;

        public ViewModelActivator Activator { get; }
    }
}