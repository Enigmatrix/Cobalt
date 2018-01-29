using System;
using System.Collections.Specialized;
using System.Globalization;
using System.Linq;
using System.Windows;
using System.Windows.Data;
using Caliburn.Micro;
using Cobalt.Common.UI.ViewModels;
using LiveCharts;
using LiveCharts.Configurations;
using LiveCharts.Wpf;

namespace Cobalt.Views.Converters
{
    public class ScopedAppDurationSeriesConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            var mapper = Mappers
                .Pie<AppDurationViewModel>()
                .Value(x => x.Duration.Ticks);

            var series = new SeriesCollection(mapper);
            if (!(value is BindableCollection<IAppDurationViewModel> coll)) return null;

            PieSeries ToSeries(AppDurationViewModel newAppDur)
            {
                return new PieSeries
                {
                    Title = newAppDur.App.Path,
                    DataLabels = true,
                    LabelPoint = LabelPoint,
                    DataLabelsTemplate = (DataTemplate) Application.Current.Resources["AppPieRepresentation"],
                    Values = new ChartValues<AppDurationViewModel>
                    {
                        newAppDur
                    }
                };
            }

            void Notify(object o, NotifyCollectionChangedEventArgs e)
            {
                if (e.Action == NotifyCollectionChangedAction.Reset)
                {
                    series.Clear();
                    series.AddRange(coll.Cast<AppDurationViewModel>().Select(ToSeries));
                }
            }


            coll.CollectionChanged += Notify;


            return series;
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
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