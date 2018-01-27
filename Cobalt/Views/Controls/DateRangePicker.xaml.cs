using System;
using System.Windows;
using MahApps.Metro.Controls;

namespace Cobalt.Views.Controls
{
    /// <summary>
    ///     Interaction logic for DateRangePicker.xaml
    /// </summary>
    public partial class DateRangePicker
    {
        public static readonly DependencyProperty StartSetProperty =
            DependencyProperty.RegisterAttached("StartSet", typeof(DateTime?), typeof(DateTimePicker),
                new PropertyMetadata(DateTime.Now, PropertyChangedCallback));

        public static readonly DependencyProperty EndSetProperty =
            DependencyProperty.RegisterAttached("EndSet", typeof(DateTime?), typeof(DateTimePicker),
                new PropertyMetadata(DateTime.Now, PropertyChangedCallback));

        public static readonly DependencyProperty StartProperty =
            DependencyProperty.Register("Start", typeof(DateTime?), typeof(DateRangePicker),
                new PropertyMetadata(DateTime.Today.Subtract(TimeSpan.FromDays(1))));

        public static readonly DependencyProperty EndProperty =
            DependencyProperty.Register("End", typeof(DateTime?), typeof(DateRangePicker),
                new PropertyMetadata(DateTime.Today));

        public DateRangePicker()
        {
            InitializeComponent();
        }

        public DateTime? Start
        {
            get => (DateTime?) GetValue(StartProperty);
            set => SetValue(StartProperty, value);
        }


        public DateTime? End
        {
            get => (DateTime?) GetValue(EndProperty);
            set => SetValue(EndProperty, value);
        }


        public DateTime? YesterdayStart { get; } = DateTime.Today.Subtract(TimeSpan.FromDays(1));
        public DateTime? YesterdayEnd { get; } = DateTime.Today;


        public DateTime? Last7DaysStart { get; } = DateTime.Today.Subtract(TimeSpan.FromDays(6));
        public DateTime? Last7DaysEnd { get; } = null;

        public static bool GetStartSet(DependencyObject obj)
        {
            return (bool) obj.GetValue(StartSetProperty);
        }

        public static void SetStartSet(DependencyObject obj, DateTime? value)
        {
            obj.SetValue(StartSetProperty, value);
        }

        public static bool GetEndSet(DependencyObject obj)
        {
            return (bool) obj.GetValue(EndSetProperty);
        }

        public static void SetEndSet(DependencyObject obj, DateTime? value)
        {
            obj.SetValue(EndSetProperty, value);
        }

        private static void PropertyChangedCallback(DependencyObject dependencyObject,
            DependencyPropertyChangedEventArgs dependencyPropertyChangedEventArgs)
        {
        }
    }
}