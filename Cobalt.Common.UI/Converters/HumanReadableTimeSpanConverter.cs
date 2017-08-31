using System;
using System.Globalization;
using System.Windows.Data;

namespace Cobalt.Common.UI.Converters
{
    public class HumanReadableTimeSpanConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            var duration = value as TimeSpan? ?? new TimeSpan();
            return (duration.Days != 0 ? $" {duration.Days}d" : "")
                   + (duration.Hours != 0 ? $" {duration.Hours}h" : "")
                   + (duration.Minutes != 0 ? $" {duration.Minutes}m" : "")
                   + (duration.Seconds != 0 ? $" {duration.Seconds}s" : "")
                   + (duration.Milliseconds != 0 ? $" {duration.Milliseconds}ms" : "");
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
}