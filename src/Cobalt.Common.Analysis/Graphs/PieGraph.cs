using System.Collections.ObjectModel;
using Cobalt.Common.Data;
using LiveChartsCore;
using LiveChartsCore.Drawing;
using LiveChartsCore.Kernel;
using LiveChartsCore.SkiaSharpView;
using LiveChartsCore.SkiaSharpView.Drawing;
using LiveChartsCore.SkiaSharpView.Drawing.Geometries;
using LiveChartsCore.SkiaSharpView.Painting;
using SkiaSharp;

namespace Cobalt.Common.Analysis.Graphs;

public class CustomDoughnutGeometry : DoughnutGeometry
{
    private SKBitmap? _resize;
    public SKBitmap? Image { get; set; }

    public override void OnDraw(SkiaSharpDrawingContext context, SKPaint paint)
    {
        base.OnDraw(context, paint);

        if (Image == null) return;

        // Calculate where to place the Image
        const float toRadians = (float)Math.PI / 180f;
        var sweepAngle = Math.Clamp(SweepAngle, 0f, 180f);
        var middleAngle = StartAngle + sweepAngle / 2f;
        var shiftDistance = InnerRadius / 2f + Width / 4f;
        var x = CenterX + shiftDistance * (float)Math.Cos(middleAngle * toRadians);
        var y = CenterY + shiftDistance * (float)Math.Sin(middleAngle * toRadians);


        // Now scale the image according to the Sweep angle
        var midArcLength = (float)Math.Sin(toRadians * sweepAngle / 2f) * shiftDistance * 2;
        var maxSizeLength = midArcLength / Math.Sqrt(2); // worst case - diagonal is perfectly on arc
        var side = maxSizeLength > 64f ? 64 : (int)maxSizeLength;
        if (side <= 0) return;
        if (_resize?.Width != side) _resize = Image.Resize(new SKSizeI(side, side), SKFilterQuality.High);

        context.Canvas.DrawBitmap(_resize, x - _resize.Height / 2f, y - _resize.Width / 2f);
    }
}

public class CustomPieSeries<T> : PieSeries<T, CustomDoughnutGeometry>
{
    protected override void OnPointCreated(ChartPoint chartPoint)
    {
        var value = chartPoint.Context.DataSource;
        var hasIcon = value as IHasIcon ?? (value as IHasEntity)?.Inner as IHasIcon;
        if (hasIcon != null)
        {
            // TODO dispose of this image
            var bitmap = SKBitmap.Decode(hasIcon.Icon);
            var visual = (CustomDoughnutGeometry)chartPoint.Context.Visual!;
            visual.Image = bitmap;
        }

        base.OnPointCreated(chartPoint);
    }
}

public class PieGraph<T> : Graph<T>
{
    // TODO move this somewhere else
    private static void MapDur(T v, ChartPoint point)
    {
        point.PrimaryValue = ((IHasDuration)v!).Duration.Ticks;
    }

    public override void SetData(IQueryable<T> data)
    {
        // replace is better than clear + add range
        var arr = data.ToArray();
        Action<T, ChartPoint>? mapping = null;
        IPaint<SkiaSharpDrawingContext>? fill = null;

        if (arr.Length == 0)
        {
            Series.Clear();
            return;
        }

        var firstElem = arr[0];
        var firstInner = (firstElem as IHasEntity)?.Inner;

        if (firstInner is IHasDuration _ || firstElem is IHasDuration _) mapping = MapDur;

        Series = new ObservableCollection<ISeries>(
            arr.Select(elem =>
            {
                // should this really check every time ... ?

                var inner = (elem as IHasEntity)?.Inner;
                var color = (elem as IHasColor)?.Color ?? (inner as IHasColor)?.Color;
                var name = (elem as IHasName)?.Name ?? (inner as IHasName)?.Name;

                if (color != null) fill = new SolidColorPaint(SKColor.Parse(color));

                return new CustomPieSeries<T>
                {
                    Values = new[] { elem },
                    Mapping = mapping,
                    Fill = fill,
                    Name = name
                };
            }));
    }
}