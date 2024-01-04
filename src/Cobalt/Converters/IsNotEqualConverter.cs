using System;
using System.Globalization;
using Avalonia.Data.Converters;

namespace Cobalt.Converters;

/// <summary>
///     Checks if the parameter is not equal to the value
/// </summary>
public class IsNotEqualConverter : IValueConverter
{
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        var eq = value?.Equals(parameter) ?? false;
        return !eq;
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        throw new InvalidOperationException();
    }
}