using System;
using System.Collections.Generic;
using System.Linq;
using System.Reactive.Linq;
using System.Text;
using System.Threading.Tasks;
using Cobalt.Common.Analysis;
using Cobalt.Common.Analysis.OutputTypes;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Repository;
using Cobalt.Common.IoC;
using Cobalt.Common.UI.Util;
using Cobalt.Common.UI.ViewModels;

namespace Cobalt.ViewModels.Extended
{
    public class ExtendedAppViewModel : AppViewModel
    {
        private IObservable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> _appHourlyChunks;

        public ExtendedAppViewModel(App app, IResourceScope scope) : base(app)
        {
            Resources = scope;
            SetStatistics();
        }

        public IResourceScope Resources { get; set; }

        public IObservable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> AppHourlyChunks
        {
            get => _appHourlyChunks;
            set => Set(ref _appHourlyChunks, value);
        }

        private void SetStatistics()
        {
            AppHourlyChunks = Resources.Resolve<IAppStatsStreamService>().GetAppUsages(DateTime.Today, DateTime.Now)
                .Where(x => x.Value.App.Path == Path)
                .SelectMany(u => SplitUsageIntoChunks(u, TimeSpan.FromHours(1), d => d.Date.AddHours(d.Hour)));
        }

        private IEnumerable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> SplitUsageIntoChunks(
            Usage<AppUsage> usage, TimeSpan chunk, Func<DateTime, DateTime> startSelector)
        {
            var appUsage = usage.Value;
            var start = appUsage.StartTimestamp;
            var end = appUsage.EndTimestamp;
            while (start < end)
            {
                var startHr = startSelector(start);
                var endHr = Min(startHr + chunk, end);
                if (!(endHr < end) && usage.JustStarted)
                    yield return new Usage<(App, DateTime, TimeSpan)>(justStarted: true,
                        value: (appUsage.App, startHr, TimeSpan.Zero));
                else
                    yield return new Usage<(App, DateTime, TimeSpan)>((appUsage.App, startHr, endHr - start));
                start = endHr;
            }
        }

        public T Min<T>(T a, T b) where T : IComparable<T>
        {
            return a.CompareTo(b) < 0 ? a : b;
        }
    }
}
