using System;
using System.Globalization;
using System.Windows.Data;
using Cobalt.Common.UI.Util;

namespace Cobalt.Common.UI.Converters
{
    public class ExePathToColorConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            return AppResourceCache.Instance.GetColor((string) value);
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
}