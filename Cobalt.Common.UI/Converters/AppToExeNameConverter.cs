using System;
using System.Diagnostics;
using System.Globalization;
using System.Windows.Data;
using Cobalt.Common.Data;

namespace Cobalt.Common.UI.Converters
{
    public class AppToExeNameConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            try
            {
                var app = (App)value;
                if (app is null) throw new NullReferenceException(nameof(App));
                return app?.Name ?? FileVersionInfo.GetVersionInfo(app.Path).FileDescription;
            }
            catch (Exception)
            {
                //todo i18nilize this
                return "Not enough access";
            }
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
}