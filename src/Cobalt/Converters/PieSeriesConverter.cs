using System;
using System.Collections;
using System.Globalization;
using System.Linq;
using System.Reactive.Linq;
using Avalonia.Data.Converters;
using Cobalt.Common.Data;
using Cobalt.Common.ViewModels;
using Humanizer;
using Humanizer.Localisation;
using LiveChartsCore.Kernel;
using LiveChartsCore.Measure;
using LiveChartsCore.SkiaSharpView;
using LiveChartsCore.SkiaSharpView.Drawing;
using LiveChartsCore.SkiaSharpView.Drawing.Geometries;
using LiveChartsCore.SkiaSharpView.Painting;
using Serilog;
using SkiaSharp;

namespace Cobalt.Converters;

/// <summary>
///     Custom <see cref="DoughnutGeometry" /> that draws icon on its slice.
/// </summary>
public class CustomDoughnutGeometry : DoughnutGeometry
{
    private SKBitmap? _resize;
    public SKBitmap? Image { get; set; }

    public override void OnDraw(SkiaSharpDrawingContext context, SKPaint paint)
    {
        base.OnDraw(context, paint);
        if (Image == null) return;

        const float toRadians = (float)Math.PI / 180f;
        const float scale = 0.9f;
        const float maxSizeLength = 64f;

        // Calculate where to place the Image
        var sweepAngle = Math.Clamp(SweepAngle, 0f, 180f);
        var middleAngle = StartAngle + sweepAngle / 2f;
        var shiftDistance = InnerRadius / 2f + Width / 4f + PushOut;
        var x = CenterX + shiftDistance * (float)Math.Cos(middleAngle * toRadians);
        var y = CenterY + shiftDistance * (float)Math.Sin(middleAngle * toRadians);


        // Now scale the image according to the Sweep angle
        var midArcLength = (float)Math.Sin(toRadians * sweepAngle / 2f) * shiftDistance * 2;
        var diagonalLength = midArcLength / Math.Sqrt(2); // worst case - diagonal is perfectly on arc
        var side = (int)(Math.Min(diagonalLength, maxSizeLength) * scale);
        if (side <= 0) return;
        if (_resize?.Width != side) _resize = Image.Resize(new SKSizeI(side, side), SKFilterQuality.High);

        context.Canvas.DrawBitmap(_resize, x - _resize.Height / 2f, y - _resize.Width / 2f);
    }

    public void DisposeImages()
    {
        Image?.Dispose();
        _resize?.Dispose();
    }
}

/// <summary>
///     Custom <see cref="PieSeries{TModel}" /> that sets the icon for the inner geometry.
/// </summary>
public class CustomPieSeries : PieSeries<object, CustomDoughnutGeometry>
{
    protected override async void OnPointCreated(ChartPoint chartPoint)
    {
        var value = chartPoint.Context.DataSource;
        var hasIcon = value as IHasIcon ?? (value as IHasInner<object>)?.Inner as IHasIcon;
        if (hasIcon != null)
            try
            {
                var icon = await hasIcon.Image.Value.FirstAsync();
                if (icon == null) return;
                var bitmap = SKBitmap.Decode(icon);
                var visual = (CustomDoughnutGeometry)chartPoint.Context.Visual!;
                visual.Image = bitmap;
            }
            catch (Exception ex)
            {
                Log.Error(ex, "OnPointCreated: set CustomPieSeries' Geometry's Image");
            }

        base.OnPointCreated(chartPoint);
    }

    // TODO this is not called?! - find another way to Dispose!
    protected override void SoftDeleteOrDisposePoint(ChartPoint point, Scaler primaryScale, Scaler secondaryScale)
    {
        (point.Context.Visual as CustomDoughnutGeometry)?.DisposeImages();
        base.SoftDeleteOrDisposePoint(point, primaryScale, secondaryScale);
    }
}

/// <summary>
///     Convert a <c>List&lt;WithDuration&lt;T&gt;&gt;</c> to a <c>List&lt;PieSeries&gt;</c>
/// </summary>
public class PieSeriesConverter : IValueConverter
{
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        return (value as IEnumerable)?.Cast<object>().Select(obj =>
        {
            var inner = (obj as IHasInner<object>)?.Inner ?? obj;
            var dur = (obj as IHasDuration)?.Duration ?? TimeSpan.Zero;
            var color = (inner as IHasColor)?.Color;
            var name = (inner as IHasName)?.Name;
            return new CustomPieSeries
            {
                Name = name,
                Values = [obj],
                // only one value - don't need ChartPoint
                Mapping = (_, idx) => new Coordinate(idx, dur.Ticks),
                // only one value - don't need ChartPoint
                ToolTipLabelFormatter = _ => dur.Humanize(2, maxUnit: TimeUnit.Day, minUnit: TimeUnit.Second),
                // Stroke = null, 
                Fill = color == null
                    ? null
                    : new SolidColorPaint(SKColor.Parse(color))
            };
        }).ToList();
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        throw new NotImplementedException();
    }
}