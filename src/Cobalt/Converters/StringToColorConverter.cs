using System;
using System.Globalization;
using Avalonia.Data.Converters;
using Avalonia.Media;

namespace Cobalt.Converters;

public class StringToColorConverter : IValueConverter
{
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        var v = value as string ?? "#000000";
        return Color.Parse(v);
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        var c = (Color?)value;
        return c?.ToString();
    }
}