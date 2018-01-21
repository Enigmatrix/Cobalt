using System;
using System.Globalization;
using System.Windows.Data;

namespace Cobalt.Common.UI.Converters
{
    public class AppIconSizeConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            var val = 100 * (double) value;
            if (val <= 1)
                return 0;
            if (val <= 3)
                return 10;
            if (val > 25 / 0.75)
                return 0.75 * val;
            return 15 + Math.Max(0, val - 15) * 6 / 11;
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
}