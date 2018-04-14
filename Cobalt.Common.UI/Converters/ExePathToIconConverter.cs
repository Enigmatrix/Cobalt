using System;
using System.Globalization;
using System.Windows.Data;
using Cobalt.Common.UI.Util;
using Cobalt.Common.UI.ViewModels;
using DColor = System.Drawing.Color;

namespace Cobalt.Common.UI.Converters
{
    public class ExePathToIconConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            return AppResourceCache.Instance.GetIcon((AppViewModel) value);
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
}