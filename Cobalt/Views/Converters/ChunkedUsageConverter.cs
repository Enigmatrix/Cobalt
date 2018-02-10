using System;
using System.Collections.Generic;
using System.Linq;
using System.Reactive.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows;
using Cobalt.Common.Analysis.OutputTypes;
using Cobalt.Common.Data;
using Cobalt.Common.IoC;
using Cobalt.Common.UI.Converters;
using Cobalt.Common.UI.Util;
using Cobalt.Common.UI.ViewModels;
using Cobalt.Common.Util;
using LiveCharts;
using LiveCharts.Configurations;
using LiveCharts.Helpers;
using LiveCharts.Wpf;

namespace Cobalt.Views.Converters
{
    public class ChunkedUsageConverter : ObservableToSeriesConverter<Usage<(App App, DateTime Time, TimeSpan Duration)>>
    {
        private static IEqualityComparer<App> PathEquality { get; }
            = new SelectorEqualityComparer<App, string>(a => a.Path);



        public DateTime Start
        {
            get => (DateTime)GetValue(StartProperty);
            set => SetValue(StartProperty, value);
        }

        public static readonly DependencyProperty StartProperty =
            DependencyProperty.Register("Start", typeof(DateTime), typeof(ChunkedUsageConverter), new PropertyMetadata(DateTime.Today));

        public DateTime End
        {
            get => (DateTime)GetValue(EndProperty);
            set => SetValue(EndProperty, value);
        }

        public static readonly DependencyProperty EndProperty =
            DependencyProperty.Register("End", typeof(DateTime), typeof(ChunkedUsageConverter), new PropertyMetadata(DateTime.Today));

        public TimeSpan Duration
        {
            get => (TimeSpan)GetValue(DurationProperty);
            set => SetValue(DurationProperty, value);
        }

        public static readonly DependencyProperty DurationProperty =
            DependencyProperty.Register("Duration", typeof(TimeSpan), typeof(ChunkedUsageConverter), new PropertyMetadata(TimeSpan.FromMilliseconds(1)));



        protected override SeriesCollection Convert(
            IObservable<Usage<(App App, DateTime Time, TimeSpan Duration)>> coll, object parameter, IResourceScope manager)
        {
            var (start, end, chunkDuration) = (Start, End, Duration);
            var count = (end - start).Ticks / chunkDuration.Ticks;
            if (count <= 0) return null;

            var mapper = Mappers
                .Xy<AppDurationViewModel>()
                .Y(x => x.Duration.Ticks);
            var series = new SeriesCollection(mapper);

            var appMap = new Dictionary<App, StackedColumnSeries>(PathEquality);

            //TODO RESOLVE DURATIONTIMER in a better way
            //var incrementor = IoCService.Instance.Resolve<IDurationIncrementor>();

            coll.ObserveOnDispatcher().Subscribe(ux =>
            {
                var x = ux.Value;
                //var justStarted = ux.JustStarted;
                if (!appMap.ContainsKey(x.App))
                {
                    var stack = new StackedColumnSeries
                    {
                        Fill = AppResourceCache.Instance.GetColor(x.App.Path),
                        Values = Enumerable.Range(0, (int)count).Select(t => new AppDurationViewModel(x.App))
                            .AsChartValues(),
                        LabelPoint = cp => x.App.Path,
                        Title = x.App.Path,
                        StrokeThickness = 0.3
                    };
                    stack.SetResourceReference(Series.StrokeProperty, "MaterialDesignBody");
                    appMap[x.App] = stack;
                    series.Add(stack);
                }

                var chunk =
                    ((ChartValues<AppDurationViewModel>) appMap[x.App].Values)[(int) ((x.Time-start).Ticks/chunkDuration.Ticks)];
                chunk.Duration += x.Duration;
                //chunk.DurationIncrement(new Usage<TimeSpan>(justStarted:justStarted, value: x.Duration), incrementor);
            }).ManageUsing(manager);


            return series;
        }
    }
}
