using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Windows.Data;
using Cobalt.Common.Data;

namespace Cobalt.Common.UI.Converters
{
    public class AppToExeNameConverter : IValueConverter
    {
        private readonly Dictionary<string, string> _nameMapper = new Dictionary<string, string>();

        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
                if (!(value is App app)) throw new NullReferenceException(nameof(App));

                if(!_nameMapper.ContainsKey(app.Path))
                    try
                    {
                        _nameMapper[app.Path] = app.Name ?? FileVersionInfo.GetVersionInfo(app.Path).FileDescription;
                    }
                    catch (FileNotFoundException)
                    {
                        //todo i18nilize this
                        _nameMapper[app.Path] = "Not enough access";
                    }

                return _nameMapper[app.Path];
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
}