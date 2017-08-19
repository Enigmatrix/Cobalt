using System;
using System.Collections.Specialized;
using System.Globalization;
using System.Windows;
using System.Windows.Data;
using Caliburn.Micro;
using Cobalt.Common.UI.ViewModels;
using LiveCharts;
using LiveCharts.Configurations;
using LiveCharts.Wpf;

namespace Cobalt.Common.UI.Converters
{
    public class AppDurationSeriesConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            var mapper = Mappers
                .Pie<AppDurationViewModel>()
                .Value(x => x.Duration.Ticks);

            var series = new SeriesCollection(mapper);
            var coll = value as BindableCollection<AppDurationViewModel>;
            if (coll == null) return null;

            void Add(AppDurationViewModel newAppDur)
            {
                series.Add(new PieSeries
                {
                    Title = newAppDur.App.Path,
                    DataLabels = true,
                    LabelPoint = LabelPoint,
                    DataLabelsTemplate = (DataTemplate) Application.Current.Resources["AppPieRepresentation"],
                    Values = new ChartValues<AppDurationViewModel>
                    {
                        newAppDur
                    }
                });
            }

            void Notify(object o, NotifyCollectionChangedEventArgs e)
            {
                //check for Action.Clear too
                if (e.Action == NotifyCollectionChangedAction.Add)
                    foreach (var appDur in e.NewItems)
                        Add((AppDurationViewModel) appDur);
                else if (e.Action == NotifyCollectionChangedAction.Reset)
                {
                    series.Clear();
                }
            };

            foreach (var appDur in coll)
                Add(appDur);

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