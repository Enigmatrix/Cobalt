using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows.Data;
using System.Windows.Media;

namespace Cobalt.Views.Converters
{
    public class ColorFromStringConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            try
            {
                return ColorConverter.ConvertFromString((string) value);
            }
            catch (Exception)
            {
                return null;
            }
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            var color = (Color) value;
            return color.A == 255 ? $"#{color.R:x2}{color.G:x2}{color.B:x2}" : $"#{color.A:x2}{color.R:x2}{color.G:x2}{color.B:x2}";
        }
    }
}
