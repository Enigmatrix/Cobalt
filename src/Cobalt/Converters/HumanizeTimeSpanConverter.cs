﻿using System;
using System.Globalization;
using Avalonia.Data.Converters;
using Humanizer;
using Humanizer.Localisation;

namespace Cobalt.Converters;

/// <summary>
///     Converts TimeSpans to human-readable formats
/// </summary>
public class HumanizeTimeSpanConverter : IValueConverter
{
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        var fullPrecision = parameter is true;
        return fullPrecision
            ? (value as TimeSpan?)?.Humanize(10, maxUnit: TimeUnit.Year, minUnit: TimeUnit.Millisecond)
            : (value as TimeSpan?)?.Humanize(2, maxUnit: TimeUnit.Day, minUnit: TimeUnit.Second);
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        throw new InvalidOperationException();
    }
}