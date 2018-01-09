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
using Cobalt.Common.Analysis.OutputTypes;
using Cobalt.Common.Data;
using Cobalt.Common.IoC;
using Cobalt.Common.UI.Util;
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
            var mapper = Mappers
                .Xy<AppDurationViewModel>()
                .Y(x => x.Duration.Ticks);
            var series = new SeriesCollection(mapper);
            if (!(value is IObservable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> coll)) return null;

            var appMap = new Dictionary<App, StackedColumnSeries>(PathEquality);

            //TODO RESOLVE DURATIONTIMER
            IDurationIncrementor incrementor = IoCService.Instance.Resolve<IDurationIncrementor>();

            coll.ObserveOnDispatcher().Subscribe(ux =>
            {
                var x = ux.Value;
                var justStarted = ux.JustStarted;
                if (!appMap.ContainsKey(x.App))
                {
                    var stack = new StackedColumnSeries
                    {
                        Values = new AppDurationViewModel[24].Select(_ => new AppDurationViewModel(x.App)).AsChartValues(),
                        LabelPoint = cp => x.App.Path
                    };
                    appMap[x.App] = stack;
                    series.Add(stack);
                }
                
                var chunk = ((ChartValues<AppDurationViewModel>) appMap[x.App].Values)[x.StartHour.Hour];
                chunk.DurationIncrement(new Usage<TimeSpan>(justStarted:justStarted, value: x.Duration), incrementor);


            });





            return series;
        }
        //TODO refactor this common componenent
         void HandleDuration(Usage<TimeSpan> d, IDurationIncrementor incrementor, IHasDuration hasDur)
        {
            if (d.JustStarted)
            {
                //handle new app/tag started here
                incrementor.Increment(hasDur);
            }
            else
            {
                incrementor.Release();
                hasDur.Duration += d.Value;
            }
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
}
