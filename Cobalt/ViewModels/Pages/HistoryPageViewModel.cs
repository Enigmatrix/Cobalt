using System;
using System.ComponentModel;
using System.Reactive;
using System.Reactive.Linq;
using Caliburn.Micro;
using Cobalt.Common.Analysis;
using Cobalt.Common.IoC;
using Cobalt.Common.UI.Util;
using Cobalt.Common.UI.ViewModels;

namespace Cobalt.ViewModels.Pages
{
    public class HistoryPageViewModel : PageViewModel
    {
        private BindableCollection<IAppDurationViewModel> _appDurations =
            new BindableCollection<IAppDurationViewModel>();

        private DateTime? _rangeEnd;
        private DateTime? _rangeStart;

        public HistoryPageViewModel(IResourceScope scope, IAppStatsStreamService stats) : base(scope)
        {
            Stats = stats;
        }

        public BindableCollection<IAppDurationViewModel> AppDurations
        {
            get => _appDurations;
            set => Set(ref _appDurations, value);
        }

        public IAppStatsStreamService Stats { get; set; }

        public DateTime? RangeStart
        {
            get => _rangeStart;
            set => Set(ref _rangeStart, value);
        }

        public DateTime? RangeEnd
        {
            get => _rangeEnd;
            set => Set(ref _rangeEnd, value);
        }

        private IObservable<EventPattern<PropertyChangedEventArgs>> PropertyChangedEvents()
        {
            return Observable.FromEventPattern<PropertyChangedEventHandler, PropertyChangedEventArgs>(
                handler => handler.Invoke, h => PropertyChanged += h, h => PropertyChanged -= h);
        }

        protected override void OnActivate(IResourceScope res)
        {
            PropertyChangedEvents()
                .Where(x =>
                    x.EventArgs.PropertyName == nameof(RangeEnd) ||
                    x.EventArgs.PropertyName == nameof(RangeStart))
                .Throttle(TimeSpan.FromMilliseconds(100))
                .Select(x => new {RangeStart, RangeEnd})
                .Subscribe(dataRange =>
                {
                    AppDurations.Clear();

                    if (dataRange.RangeStart == null && dataRange.RangeEnd == null) return;

                    var stats = res.Resolve<IAppStatsStreamService>();
                    var appDurationsStream =
                        stats.GetAppDurations(dataRange.RangeStart ?? DateTime.MinValue,
                            dataRange.RangeEnd); //.Publish();


                    appDurationsStream
                        .Select(x =>
                        {
                            var appDur = new AppDurationViewModel(x.App);

                            x.Duration
                                .Subscribe(d =>
                                {
                                    if(!d.JustStarted)
                                        appDur.Duration += d.Value;
                                })
                                .ManageUsing(res);

                            return appDur;
                        })
                        .Buffer(TimeSpan.FromMilliseconds(50))
                        .Subscribe(x =>
                        {
                            if(x.Count != 0)
                                AppDurations.AddRange(x);
                        }).ManageUsing(res);
                });
        }

        protected override void OnDeactivate(bool close, IResourceScope res)
        {
                    AppDurations.Clear();
        }
    }
}