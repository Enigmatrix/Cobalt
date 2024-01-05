using System;
using System.Collections.Generic;
using System.Globalization;
using Avalonia.Data.Converters;

namespace Cobalt.Converters;

/// <summary>
///     Converts from percentage and control width to a number representing it's left position
/// </summary>
public class PercentToLeftConverter : IMultiValueConverter
{
    public object? Convert(IList<object?> values, Type targetType, object? parameter, CultureInfo culture)
    {
        if (values.Count != 2) return null;
        if (values[0] is not double percent || values[1] is not double width) return null;

        return width * percent / 100.0;
    }
}