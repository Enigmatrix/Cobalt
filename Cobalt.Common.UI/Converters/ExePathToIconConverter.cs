using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Drawing;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Windows;
using System.Windows.Data;
using System.Windows.Interop;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Xml.Linq;
using Cobalt.Common.UI.Util;
using ColorThiefDotNet;
using LiveCharts.Wpf;
using static System.Convert;
using Color = System.Windows.Media.Color;
using DColor = System.Drawing.Color;

namespace Cobalt.Common.UI.Converters
{
    public class ExePathToIconConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            return AppResourceCache.Instance.GetIcon(value as string);
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
}