using System;
using System.Globalization;
using System.Windows;
using System.Windows.Data;
using Cobalt.Common.IoC;

namespace Cobalt.Common.UI.Converters
{
    public abstract class ObservableConverter<T, TE> : DependencyObject, IMultiValueConverter
    {
        public static readonly DependencyProperty BufferDurationProperty =
            DependencyProperty.Register("BufferDuration", typeof(TimeSpan), typeof(ObservableConverter<T, TE>),
                new PropertyMetadata(TimeSpan.Zero));

        public TimeSpan BufferDuration
        {
            get => (TimeSpan) GetValue(BufferDurationProperty);
            set => SetValue(BufferDurationProperty, value);
        }

        public virtual object Convert(object[] values, Type targetType, object parameter, CultureInfo culture)
        {
            if (values.Length != 2 ||
                !(values[0] is IObservable<T> coll) ||
                !(values[1] is IResourceScope manager)) return null;
            return Convert(coll, parameter, manager);
        }

        //usually no need to convert back
        public virtual object[] ConvertBack(object value, Type[] targetTypes, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }

        protected abstract TE Convert(IObservable<T> values, object parameter, IResourceScope manager);
    }
}