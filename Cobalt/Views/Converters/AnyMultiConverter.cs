using System;
using System.Globalization;
using System.Linq;
using System.Windows.Data;

namespace Cobalt.Views.Converters
{
    public class AnyMultiConverter : IMultiValueConverter
    {
        public object Convert(object[] values, Type targetType, object parameter, CultureInfo culture)
        {
            return values.Any(v => (bool) v);
        }

        public object[] ConvertBack(object value, Type[] targetTypes, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
}