using System;
using System.Globalization;
using System.Linq;
using System.Reactive.Linq;
using System.Windows;
using System.Windows.Data;
using Cobalt.Common.IoC;
using Cobalt.Common.UI.ViewModels;
using LiveCharts;
using LiveCharts.Configurations;
using LiveCharts.Wpf;

namespace Cobalt.Common.UI.Converters
{
    public class AppDurationSeriesConverter : DependencyObject, IMultiValueConverter
    {
        public static readonly DependencyProperty BufferDurationProperty =
            DependencyProperty.Register("BufferDuration", typeof(TimeSpan), typeof(AppDurationSeriesConverter),
                new PropertyMetadata(TimeSpan.Zero));


        public TimeSpan BufferDuration
        {
            get => (TimeSpan) GetValue(BufferDurationProperty);
            set => SetValue(BufferDurationProperty, value);
        }


        public object Convert(object[] values, Type targetType, object parameter, CultureInfo culture)
        {
            if (values.Length != 2 ||
                !(values[0] is IObservable<IAppDurationViewModel> coll) ||
                !(values[1] is IResourceScope manager)) return null;

            var mapper = Mappers
                .Pie<IAppDurationViewModel>()
                .Value(x => x.Duration.Ticks);
            var series = new SeriesCollection(mapper);

            PieSeries ToSeries(IAppDurationViewModel newAppDur)
            {
                return new PieSeries
                {
                    Title = newAppDur.App.Path,
                    DataLabels = true,
                    LabelPoint = LabelPoint,
                    DataLabelsTemplate = (DataTemplate) Application.Current.Resources["AppPieRepresentation"],
                    Values = new ChartValues<IAppDurationViewModel>
                    {
                        newAppDur
                    }
                };
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

        public object[] ConvertBack(object value, Type[] targetTypes, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }

        private string LabelPoint(ChartPoint c)
        {
            var duration = (c.Instance as IAppDurationViewModel)?.Duration;
            return duration?.ToString(@"hh\:mm\:ss\.fff") ?? "";
        }
    }
}