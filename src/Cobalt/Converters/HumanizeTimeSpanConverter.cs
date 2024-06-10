using System;
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
        // TODO inhumanly beat Humanizer till it shortens it like this.
        var fullPrecision = parameter is true;
        var ret = fullPrecision
            ? (value as TimeSpan?)?.Humanize(10, maxUnit: TimeUnit.Year, minUnit: TimeUnit.Millisecond)
            : (value as TimeSpan?)?.Humanize(2, maxUnit: TimeUnit.Day, minUnit: TimeUnit.Second);
        return ret?
            .Replace("years", "y").Replace("year", "y")
            .Replace("months", "m").Replace("month", "m")
            .Replace("weeks", "w").Replace("week", "w")
            .Replace("days", "d").Replace("day", "d")
            .Replace("hours", "h").Replace("hour", "h")
            .Replace("minutes", "m").Replace("minute", "m")
            .Replace("seconds", "s").Replace("second", "s")
            .Replace("milliseconds", "ms").Replace("millisecond", "ms");
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        throw new InvalidOperationException();
    }
}