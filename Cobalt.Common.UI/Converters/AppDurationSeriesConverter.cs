using System;
using System.Collections.Generic;
using System.Collections.Specialized;
using System.Globalization;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
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
                    Values = new ChartValues<AppDurationViewModel>
                    {
                        newAppDur
                    }
                });
            }

            foreach (var appDur in coll)
            {
                Add(appDur);
            }
            coll.CollectionChanged += (_, e) =>
            {
                //check for Action.Clear too
                if (e.Action == NotifyCollectionChangedAction.Add)
                {
                    foreach (var appDur in e.NewItems)
                    {
                        Add((AppDurationViewModel)appDur);
                    }
                }
            };

            return series;
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
}
