using System;
using System.Globalization;
using System.Windows.Data;
using Cobalt.Views.Controls;

namespace Cobalt.Views.Converters
{
    public class AppMenuItemToTypeConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            return (value as AppMenuItem)?.Type;
        }
    }
}