using System;
using System.Collections.Generic;
using System.Collections.Specialized;
using System.Globalization;
using System.Linq;
using System.Reactive.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Data;
using Caliburn.Micro;
using Cobalt.Common.Data;
using Cobalt.Common.UI.ViewModels;
using Cobalt.Common.Util;
using LiveCharts;
using LiveCharts.Configurations;
using LiveCharts.Helpers;
using LiveCharts.Wpf;

namespace Cobalt.Views.Converters
{
    public class HourlyUsageConverter : IValueConverter
    {
        private static IEqualityComparer<App> PathEquality { get; }
            = new SelectorEqualityComparer<App, string>(a => a.Path);

        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            var mapper = Mappers.Xy<TimeSpan>()
                .Y(x => x.Ticks);
            var series = new SeriesCollection(mapper);
            if (!(value is IObservable<(App App, DateTime StartHour, TimeSpan Duration)> coll)) return null;

            void Add((App App, DateTime StartHour, TimeSpan Duration) appDur)
            {
                series.Add(new PieSeries
                {
                });
            }

            var appMap = new Dictionary<App, StackedColumnSeries>(PathEquality);

            coll.ObserveOnDispatcher().Subscribe(x =>
            {
                if (!appMap.ContainsKey(x.App))
                {
                    var stack = new StackedColumnSeries
                    {
                        Values = new TimeSpan[24].AsChartValues(),
                        LabelPoint = cp => x.App.Path
                    };
                    appMap[x.App] = stack;
                    series.Add(stack);
                }
                ((ChartValues<TimeSpan>)appMap[x.App].Values)[x.StartHour.Hour] += x.Duration;
                
            });


            return series;
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
}
