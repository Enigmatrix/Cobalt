﻿using System;
using System.Collections.Generic;
using System.Linq;
using System.Reactive.Linq;
using System.Windows.Media;
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
    public class
        PerWeekUsageConverter : ObservableToSeriesConverter<Usage<(App App, DateTime StartDay, TimeSpan Duration)>>
    {
        private static IEqualityComparer<App> PathEquality { get; }
            = new SelectorEqualityComparer<App, string>(a => a.Path);


        protected override SeriesCollection Convert(
            IObservable<Usage<(App App, DateTime StartDay, TimeSpan Duration)>> coll, IResourceScope manager)
        {
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
                        Values = new AppDurationViewModel[7].Select(_ => new AppDurationViewModel(x.App))
                            .AsChartValues(),
                        LabelPoint = cp => x.App.Path,
                        Title = x.App.Path,
                        StrokeThickness = 0.3
                    };
                    stack.SetResourceReference(Series.StrokeProperty, "MaterialDesignBody");
                    appMap[x.App] = stack;
                    series.Add(stack);
                }

                var chunk = ((ChartValues<AppDurationViewModel>) appMap[x.App].Values)[(int) x.StartDay.DayOfWeek];
                chunk.Duration += x.Duration;
                //chunk.DurationIncrement(new Usage<TimeSpan>(justStarted:justStarted, value: x.Duration), incrementor);
            }).ManageUsing(manager);


            return series;
        }
    }
}