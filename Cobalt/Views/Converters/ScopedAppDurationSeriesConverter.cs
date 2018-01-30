using System;
using System.Collections.Specialized;
using System.Globalization;
using System.Linq;
using System.Reactive.Linq;
using System.Windows;
using System.Windows.Data;
using System.Windows.Threading;
using Caliburn.Micro;
using Cobalt.Common.IoC;
using Cobalt.Common.UI.ViewModels;
using LiveCharts;
using LiveCharts.Configurations;
using LiveCharts.Wpf;

namespace Cobalt.Views.Converters
{
    public class ScopedAppDurationSeriesConverter : DependencyObject, IValueConverter
    {

        public IResourceScope Manager
        {
            get => (IResourceScope)GetValue(ManagerProperty);
            set => SetValue(ManagerProperty, value);
        }

        public static readonly DependencyProperty ManagerProperty =
            DependencyProperty.Register("Manager", typeof(IResourceScope), typeof(ScopedAppDurationSeriesConverter), new PropertyMetadata(null));

        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (!(value is IObservable<IAppDurationViewModel> coll) || Manager == null) return null;

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
                    DataLabelsTemplate = (DataTemplate) Application.Current.Resources["ScopedAppPieRepresentation"],
                    Values = new ChartValues<IAppDurationViewModel>
                    {
                        newAppDur
                    }
                };
            }

            coll.Buffer(TimeSpan.FromMilliseconds(100))
                .Where(x => x.Count != 0)
                .ObserveOnDispatcher()
                .Subscribe(x => series.AddRange(x.Select(ToSeries)))
                .ManageUsing(Manager);

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