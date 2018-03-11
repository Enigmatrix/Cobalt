using System;
using System.Globalization;
using System.Windows.Data;

namespace Cobalt.Common.UI.Converters
{
    public class AppIconSizeConverter : IMultiValueConverter
    {
        public object Convert(object[] values, Type targetType, object parameter, CultureInfo culture)
        {
            if (!(values[0] is double participation) || !(values[1] is double height) ||
                !(values[2] is double width)) return null;
            var minsz = Min(height, width);
            var transed = Min(0.15, participation * 1.2);
            return transed * minsz;
        }

        public object[] ConvertBack(object value, Type[] targetTypes, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }

        public static double Min(double a, double b)
        {
            return a < b ? a : b;
        }

        public static double Max(double a, double b)
        {
            return a > b ? a : b;
        }
    }
}