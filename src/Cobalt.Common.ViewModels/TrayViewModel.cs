using System;
using System.Collections.ObjectModel;
using System.Reactive.Linq;
using Cobalt.Common.Data;
using Cobalt.Common.ViewModels.Entities;
using Cobalt.Common.ViewModels.Statistics;
using DynamicData;

namespace Cobalt.Common.ViewModels
{
    public class TrayViewModel : ViewModelBase, IDisposable
    {
        private readonly ReadOnlyObservableCollection<AlertViewModel> _alerts;
        private readonly IDisposable _alertsBinding;
        private readonly ReadOnlyObservableCollection<AppDurationViewModel> _appDurations;

        private readonly IDisposable _appDurationsBinding;


        public TrayViewModel(IAnalyzer analyzer, IDatabase db, IEntityManager mgr)
        {
            _appDurationsBinding = analyzer.AppDurations(
                    new DateTimeRange(DateTimeBound.NewBound(DateTime.Today), DateTimeBound.Unbounded),
                    IdleOptions.Irrelevant)
                .Select(x =>
                    new ChangeSet<AppDurationViewModel>(new[]
                        {new Change<AppDurationViewModel>(ListChangeReason.Add, x)}))
                .ObserveOnDispatcher()
                .Bind(out _appDurations)
                .DisposeMany()
                .Subscribe();

            _alertsBinding = db.Alerts()
                .Select(mgr.GetAlert)
                .ToObservableChangeSet()
                .Bind(out _alerts)
                .DisposeMany()
                .Subscribe();
        }

        public ReadOnlyObservableCollection<AppDurationViewModel> AppDurations => _appDurations;

        public ReadOnlyObservableCollection<AlertViewModel> Alerts => _alerts;

        public void Dispose()
        {
            _appDurationsBinding.Dispose();
            _alertsBinding.Dispose();
        }
    }
}