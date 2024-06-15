using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.Input;
using Avalonia.LogicalTree;
using Cobalt.Common.Data;
using Cobalt.Common.ViewModels;
using Cobalt.Common.ViewModels.Analysis;
using LiveChartsCore;
using LiveChartsCore.Kernel;
using LiveChartsCore.Kernel.Sketches;
using LiveChartsCore.SkiaSharpView.Avalonia;
using LiveChartsCore.SkiaSharpView.Drawing;

namespace Cobalt.Charts;

public record class TooltipPoint(object Model, bool Selected)
{
    public TimeSpan Duration => ((IHasDuration)Model).Duration;
    public string Name => ((IHasName)((IHasInner<object>)Model).Inner).Name;
    public Query<byte[]?> Icon => ((IHasIcon)((IHasInner<object>)Model).Inner).Image;
}

public record class TooltipContext(IEnumerable<TooltipPoint> Points, int Extra);

public partial class ChartTooltip : UserControl, IChartTooltip<SkiaSharpDrawingContext>
{
    private const int TakePoints = 14;

    protected static MethodInfo HandlePositionChange =
        typeof(Popup).GetMethod("HandlePositionChange", BindingFlags.Instance | BindingFlags.NonPublic) ??
        throw new Exception("no HandlePositionChange method found on Popup");

    private Control? _chartControl;
    private Popup? _mainPopup;
    private Panel? _panel;

    public ChartTooltip()
    {
        InitializeComponent();
    }

    public void Show(IEnumerable<ChartPoint> foundPoints, Chart<SkiaSharpDrawingContext> chart)
    {
        if (_mainPopup == null)
        {
            _chartControl = (PieChart)chart.View;
            _chartControl.PointerMoved += OnChartControlOnPointerMoved;
            _panel = _chartControl.FindLogicalAncestorOfType<Panel>()!;

            _mainPopup = new Popup
            {
                WindowManagerAddShadowHint = false,
                PlacementTarget = _chartControl,
                Placement = PlacementMode.Pointer,
                Width = 250,
                MaxHeight = 350,
                HorizontalOffset = 10,
                VerticalOffset = 10,
                Child = this
            };

            // popup needs a visual parent
            _panel.Children.Add(_mainPopup);
        }

        var allPoints = chart.Series.SelectMany(series => series.Values?.Cast<object>() ?? []).ToList();
        // not sure why >1 value is chosen.
        var chosenPoints = foundPoints.SelectMany(series => series.Context.Series.Values?.Cast<object>() ?? [])
            .ToList();
        var points = chosenPoints
            .Select(model => new TooltipPoint(model, true))
            .Concat(allPoints
                .Except(chosenPoints)
                .Select(model => new TooltipPoint(model, false))
                .OrderByDescending(point => point.Duration))
            .ToList();

        // some elements get cut off so I am just going to limit the length
        DataContext = new TooltipContext(points.Take(TakePoints).ToList(), points.Count - TakePoints);

        _mainPopup.Open();
    }

    public void Hide(Chart<SkiaSharpDrawingContext> chart)
    {
        _mainPopup?.Close();
    }

    private void OnChartControlOnPointerMoved(object? sender, PointerEventArgs args)
    {
        if (_mainPopup == null) return;
        HandlePositionChange.Invoke(_mainPopup, []);
    }
}