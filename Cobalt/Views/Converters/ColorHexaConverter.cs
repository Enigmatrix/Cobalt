using System;
using System.Globalization;
using System.Windows.Data;
using System.Windows.Media;
using Cobalt.Views.Controls;

namespace Cobalt.Views.Converters
{
    public class ColorHexaConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            var color = (ColorPicker.ColorInternal) value;
            return color != null ? $"#{color.A:X2}{color.R:X2}{color.G:X2}{color.B:X2}" : "";
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            return ColorPicker.ColorInternal.FromColor((Color) ColorConverter.ConvertFromString((string)value));
        }
    }
}