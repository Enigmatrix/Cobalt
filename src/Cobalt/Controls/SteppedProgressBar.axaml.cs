using System;
using System.Collections.Generic;
using System.Globalization;
using Avalonia.Controls;
using Avalonia.Data.Converters;

namespace Cobalt.Controls;

public partial class SteppedProgressBar : UserControl
{
    public SteppedProgressBar()
    {
        InitializeComponent();
    }

    public IEnumerable<IWithProgress> ItemsSource { get; } = new WithProgressClass[]
    {
        new("0", 0),
        new("A", 0.10),
        new("B", 0.5),
        new("1", 1)
    };

    public record WithProgressClass(string Text, double Percent) : IWithProgress;

    public interface IWithProgress
    {
        double Percent { get; }
    }
}

public class PercentToLeftConverter : IMultiValueConverter
{
    public object? Convert(IList<object?> values, Type targetType, object? parameter, CultureInfo culture)
    {
        if (values.Count != 2) return null;
        if (values[0] is not double percent || values[1] is not double width) return null;

        return width * percent / 100.0;
    }
}