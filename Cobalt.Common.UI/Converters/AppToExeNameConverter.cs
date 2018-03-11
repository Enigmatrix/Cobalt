using System;
using System.Globalization;
using System.Windows.Data;
using Cobalt.Common.UI.Util;
using Cobalt.Common.UI.ViewModels;

namespace Cobalt.Common.UI.Converters
{
    public class AppToExeNameConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            switch (value)
            {
                case AppViewModel app:
                    return app.Name ?? AppResourceCache.Instance.GetName(app.Path);
                case string path:
                    return AppResourceCache.Instance.GetName(path);
                default: throw new ArgumentException();
            }
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
}