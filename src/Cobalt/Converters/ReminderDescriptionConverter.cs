using System;
using System.Collections.Generic;
using System.Globalization;
using Avalonia.Data.Converters;
using Humanizer;
using Humanizer.Localisation;

namespace Cobalt.Converters;

/// <summary>
///     Converts Usage Limit and Threshold value to a Reminder description
/// </summary>
public class ReminderDescriptionConverter : IMultiValueConverter
{
    public object? Convert(IList<object?> values, Type targetType, object? parameter, CultureInfo culture)
    {
        if (values.Count != 2) return null;
        if (values[0] is not double threshold || values[1] is not TimeSpan usageLimit) return null;
        var duration = new TimeSpan((long)(usageLimit.Ticks * threshold));
        var durationString = duration.Humanize(2, maxUnit: TimeUnit.Day, minUnit: TimeUnit.Second);
        return $"{durationString} ({threshold}%)";
    }
}