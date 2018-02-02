using System;
using System.Linq;
using System.Reactive.Linq;
using System.Windows;
using System.Windows.Media;
using Cobalt.Common.IoC;
using Cobalt.Common.UI.Util;
using Cobalt.Common.UI.ViewModels;
using LiveCharts;
using LiveCharts.Configurations;
using LiveCharts.Wpf;

namespace Cobalt.Common.UI.Converters
{
    public class AppDurationSeriesConverter : ObservableToSeriesConverter<IAppDurationViewModel>
    {
        protected override SeriesCollection Convert(IObservable<IAppDurationViewModel> coll, IResourceScope manager)
        {
            var mapper = Mappers
                .Pie<AppDurationViewModel>()
                .Value(x => x.Duration.Ticks);
            var series = new SeriesCollection(mapper);

            PieSeries ToSeries(AppDurationViewModel newAppDur)
            {
                return new PieSeries
                {
                    Title = newAppDur.App.Path,
                    Fill = AppResourceCache.Instance.GetColor(newAppDur.App.Path),
                    DataLabels = true,
                    LabelPoint = LabelPoint,
                    DataLabelsTemplate = (DataTemplate) Application.Current.Resources["AppPieRepresentation"],
                    Values = new ChartValues<AppDurationViewModel>
                    {
                        newAppDur
                    }
                };
            }

            var sub = BufferDuration == TimeSpan.Zero
                ? coll.ObserveOnDispatcher().Subscribe(x => series.Add(ToSeries((AppDurationViewModel) x)))
                : coll.Buffer(BufferDuration)
                    .Where(x => x.Count != 0)
                    .ObserveOnDispatcher()
                    .Subscribe(x => series.AddRange(x.Cast<AppDurationViewModel>().Select(ToSeries)));

            sub.ManageUsing(manager);
            return series;
        }

        private string LabelPoint(ChartPoint c)
        {
            var duration = (c.Instance as IAppDurationViewModel)?.Duration;
            return duration?.ToString(@"hh\:mm\:ss\.fff") ?? "";
        }
    }
}