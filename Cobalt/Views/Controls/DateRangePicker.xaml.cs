using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;
using System.Windows.Documents;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Navigation;
using System.Windows.Shapes;
using MahApps.Metro.Controls;

namespace Cobalt.Views.Controls
{
    /// <summary>
    /// Interaction logic for DateRangePicker.xaml
    /// </summary>
    public partial class DateRangePicker
    {
        public DateRangePicker()
        {
            InitializeComponent();
        }


        public static readonly DependencyProperty StartSetProperty =
            DependencyProperty.RegisterAttached("StartSet", typeof(DateTime?), typeof(DateTimePicker),
                new PropertyMetadata(DateTime.Now, PropertyChangedCallback));

        public static bool GetStartSet(DependencyObject obj)
        {
            return (bool) obj.GetValue(StartSetProperty);
        }

        public static void SetStartSet(DependencyObject obj, DateTime? value)
        {
            obj.SetValue(StartSetProperty, value);
        }

        public static readonly DependencyProperty EndSetProperty =
            DependencyProperty.RegisterAttached("EndSet", typeof(DateTime?), typeof(DateTimePicker),
                new PropertyMetadata(DateTime.Now, PropertyChangedCallback));

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

        public DateTime? Start
        {
            get => (DateTime?)GetValue(StartProperty);
            set => SetValue(StartProperty, value);
        }

        public static readonly DependencyProperty StartProperty =
            DependencyProperty.Register("Start", typeof(DateTime?), typeof(DateRangePicker), new PropertyMetadata(DateTime.Today.Subtract(TimeSpan.FromDays(1))));


        public DateTime? End
        {
            get => (DateTime?)GetValue(EndProperty);
            set => SetValue(EndProperty, value);
        }

        public static readonly DependencyProperty EndProperty =
            DependencyProperty.Register("End", typeof(DateTime?), typeof(DateRangePicker), new PropertyMetadata(DateTime.Today));


        public DateTime? YesterdayStart { get; } = DateTime.Today.Subtract(TimeSpan.FromDays(1));
        public DateTime? YesterdayEnd { get; } = DateTime.Today;
        

        public DateTime? Last7DaysStart { get; } = DateTime.Today.Subtract(TimeSpan.FromDays(6));
        public DateTime? Last7DaysEnd { get; } = null;

    }
}
