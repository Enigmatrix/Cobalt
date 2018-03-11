using System;
using System.Linq;
using System.Reactive.Linq;
using System.Windows;
using Cobalt.Common.IoC;
using Cobalt.Common.UI.Util;
using Cobalt.Common.UI.ViewModels;
using LiveCharts;
using LiveCharts.Configurations;
using LiveCharts.Wpf;

namespace Cobalt.Common.UI.Converters
{
    public class TagDurationSeriesConverter : ObservableConverter<TagDurationViewModel, SeriesCollection>
    {
        protected override SeriesCollection Convert(IObservable<TagDurationViewModel> coll, object p,
            IResourceScope manager)
        {
            var mapper = Mappers
                .Pie<TagDurationViewModel>()
                .Value(x => x.Duration.Ticks);
            var series = new SeriesCollection(mapper);

            PieSeries ToSeries(TagDurationViewModel newAppDur)
            {
                var slice = new PieSeries
                {
                    Title = newAppDur.Tag.Name,
                    StrokeThickness = .5,
                    DataLabels = true,
                    LabelPoint = LabelPoint,
                    DataLabelsTemplate = (DataTemplate) Application.Current.Resources["TagPieRepresentation"],
                    Values = new ChartValues<TagDurationViewModel>
                    {
                        newAppDur
                    }
                };
                slice.SetResourceReference(Series.StrokeProperty, "MaterialDesignBody");
                return slice;
            }

            var sub = BufferDuration == TimeSpan.Zero
                ? coll.ObserveOnDispatcher().Subscribe(x => series.Add(ToSeries(x)))
                : coll.Buffer(BufferDuration)
                    .Where(x => x.Count != 0)
                    .ObserveOnDispatcher()
                    .Subscribe(x => series.AddRange(x.Select(ToSeries)));

            sub.ManageUsing(manager);
            return series;
        }

        private string LabelPoint(ChartPoint c)
        {
            var duration = (c.Instance as AppDurationViewModel)?.Duration;
            return duration?.ToString(@"hh\:mm\:ss\.fff") ?? "";
        }
    }
}