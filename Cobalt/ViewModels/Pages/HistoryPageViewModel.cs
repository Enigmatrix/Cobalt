using System;
using System.ComponentModel;
using System.Reactive;
using System.Reactive.Linq;
using Cobalt.Common.Analysis;
using Cobalt.Common.IoC;
using Cobalt.Common.UI.ViewModels;

namespace Cobalt.ViewModels.Pages
{
    public class HistoryPageViewModel : PageViewModel
    {
        private DateTime? _rangeStart;
        private DateTime? _rangeEnd;

        public HistoryPageViewModel(IResourceScope scope)
        {
        }

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

        protected override void OnActivate()
        {
            PropertyChangedEvents()
                .Where(x =>
                    x.EventArgs.PropertyName == nameof(RangeEnd) ||
                    x.EventArgs.PropertyName == nameof(RangeStart))
                .Throttle(TimeSpan.FromMilliseconds(100))
                .Select(x => new {RangeStart, RangeEnd})
                .Subscribe(dataRange =>
                {
                    //work here
                });
        }

        protected override void OnDeactivate(bool close)
        {
        }
    }
}