﻿using System;
using System.Globalization;
using System.Linq;
using System.Windows.Data;

namespace Cobalt.Views.Converters
{
    public class BothMultiConverter : IMultiValueConverter
    {
        public object Convert(object[] values, Type targetType, object parameter, CultureInfo culture)
        {
            return values.All(v => (bool) v);
        }

        public object[] ConvertBack(object value, Type[] targetTypes, object parameter, CultureInfo culture)
        {
            return new object[] {false, true};
        }
    }
}
