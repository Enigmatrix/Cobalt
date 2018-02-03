using System;
using System.Globalization;
using System.Windows;
using System.Windows.Data;
using LiveCharts;

namespace Cobalt.Views.Converters
{
    public class BooleanToLegendLocation : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            return ((bool) value) ? LegendLocation.Right : LegendLocation.None;
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
}