using System;
using System.Reactive.Linq;
using Cobalt.Common.Analysis;
using Cobalt.Common.IoC;
using Cobalt.Common.UI.ViewModels;
using Cobalt.Common.Util;

namespace Cobalt.ViewModels.Pages
{
    public class HistoryPageViewModel : PageViewModel
    {

        public HistoryPageViewModel(IResourceScope scope, IAppStatsStreamService stats) : base(scope)
        {
            Stats = stats;
        }

        public IObservable<AppDurationViewModel> AppDurations { get; set; }

        public IAppStatsStreamService Stats { get; set; }

        public DateTime? RangeStart { get; set; }

        public DateTime? RangeEnd { get; set; }

        protected override void OnActivate(IResourceScope res)
        {
            this.PropertyChanges()
                .Where(x =>
                    x == nameof(RangeEnd) ||
                    x == nameof(RangeStart))
                .Throttle(TimeSpan.FromMilliseconds(100))
                .Select(x => new {RangeStart, RangeEnd})
                .Subscribe(dataRange =>
                {
                    if (dataRange.RangeStart == null && dataRange.RangeEnd == null) return;

                    var stats = res.Resolve<IAppStatsStreamService>();
                    var appDurationsStream =
                        stats.GetAppDurations(dataRange.RangeStart ?? DateTime.MinValue,
                            dataRange.RangeEnd); //.Publish();


                    AppDurations = appDurationsStream
                        .Select(x =>
                        {
                            var appDur = new AppDurationViewModel(x.App);

                            x.Duration
                                .Subscribe(d =>
                                {
                                    if (!d.JustStarted)
                                        appDur.Duration += d.Value;
                                })
                                .ManageUsing(res);

                            return appDur;
                        });
                }).ManageUsing(res);
        }

        protected override void OnDeactivate(bool close, IResourceScope res)
        {
        }
    }
}