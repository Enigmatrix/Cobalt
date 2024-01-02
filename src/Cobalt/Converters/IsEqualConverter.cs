using System;
using System.Globalization;
using Avalonia.Data.Converters;

namespace Cobalt.Converters;

/// <summary>
///     Checks if the parameter is equal to the value
/// </summary>
public class IsEqualConverter : IValueConverter
{
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        return value?.Equals(parameter) ?? false;
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        throw new InvalidOperationException();
    }
}