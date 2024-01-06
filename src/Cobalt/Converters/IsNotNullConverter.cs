using System;
using System.Globalization;
using Avalonia.Data.Converters;

namespace Cobalt.Converters;

/// <summary>
///     Checks if the value is not null
/// </summary>
public class IsNotNullConverter : IValueConverter
{
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        return value != null;
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        throw new InvalidOperationException();
    }
}