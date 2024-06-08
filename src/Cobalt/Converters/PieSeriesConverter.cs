using System;
using System.Collections;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using Avalonia.Data.Converters;
using Cobalt.Common.Data;
using Humanizer;
using Humanizer.Localisation;
using LiveChartsCore.Kernel;
using LiveChartsCore.Measure;
using LiveChartsCore.SkiaSharpView;
using LiveChartsCore.SkiaSharpView.Painting;
using SkiaSharp;

namespace Cobalt.Converters;

// TODO
public class PieSeriesConverter : IValueConverter
{
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {

        /*foreach (var o in (IEnumerable)value)
        {
            var hasDur = o as IHasDuration;
            var hasInner = o as IHasInner<object>;
        }*/

        return ((IEnumerable)value).Cast<object>().Select(obj =>
        {
            var inner = (obj as IHasInner<object>)?.Inner ?? obj;
            var dur = (obj as IHasDuration)?.Duration ?? TimeSpan.Zero;
            var color = (inner as IHasColor)?.Color;
            var name = (inner as IHasName)?.Name;
            return new PieSeries<object>
            {
                Name = name,
                Values = [obj],
                // only 1 object, so don't care
                Mapping = (_, idx) => new(idx, dur.Ticks),
                // DataLabelsFormatter = null,
                // IsHoverable = false,
                // IsVisibleAtLegend = false,
                /*WhenPointMeasured = point =>
                {

                },*/
                // MiniatureShapeSize = 0,
                // DataLabelsPaint = null,
                // don't really need to reference the ChartPoint, only one value
                ToolTipLabelFormatter = _ => dur.Humanize(2, maxUnit: TimeUnit.Day, minUnit: TimeUnit.Second),
                // Stroke = null, 
                Fill = color == null
                    ? null
                    : new SolidColorPaint(SKColor.Parse(color)),
                //DataLabelsPosition = PolarLabelsPosition.ChartCenter
            };
        }).ToList();
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        throw new NotImplementedException();
    }
}