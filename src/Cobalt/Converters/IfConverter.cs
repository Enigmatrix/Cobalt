using System;
using System.Collections.Generic;
using System.Globalization;
using Avalonia.Data.Converters;

namespace Cobalt.Converters;

/// <summary>
///     Converts from percentage and control width to a number representing it's left position
/// </summary>
public class IfConverter : IMultiValueConverter
{
    public object? Convert(IList<object?> values, Type targetType, object? parameter, CultureInfo culture)
    {
        if (values.Count != 3) return null;
        if (values[0] is not bool cond) return null;

        return cond ? values[1] : values[2];
    }
}