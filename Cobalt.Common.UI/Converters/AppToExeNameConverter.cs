using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Windows.Data;
using Cobalt.Common.Data;
using Cobalt.Common.UI.Util;

namespace Cobalt.Common.UI.Converters
{
    public class AppToExeNameConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (!(value is App app)) throw new NullReferenceException(nameof(App));
            return app.Name ?? AppResourceCache.Instance.GetName(app.Path);
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
}