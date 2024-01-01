using System;
using System.Globalization;
using Avalonia.Data.Converters;
using Avalonia.Media;
using Avalonia.Media.Immutable;

namespace Cobalt.Converters;

/// <summary>
///     Converts from hex-string color e.g. #123456 to Avalonia <see cref="ImmutableSolidColorBrush" />
/// </summary>
public class ColorStringToBrushConverter : IValueConverter
{
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        var colorStr = (string?)value;
        if (colorStr == null) return null;
        return Color.TryParse(colorStr, out var color) ? new ImmutableSolidColorBrush(color) : null;
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        throw new InvalidOperationException();
    }
}