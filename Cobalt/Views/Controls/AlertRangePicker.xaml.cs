using System.Windows;
using Cobalt.Common.Data;

namespace Cobalt.Views.Controls
{
    /// <summary>
    ///     Interaction logic for AlertRangePicker.xaml
    /// </summary>
    public partial class AlertRangePicker
    {
        public static readonly DependencyProperty AlertRangeProperty =
            DependencyProperty.Register("AlertRange", typeof(AlertRange), typeof(AlertRangePicker),
                new PropertyMetadata(null, RangeChangeCallback));

        public AlertRangePicker()
        {
            InitializeComponent();
        }


        public AlertRange AlertRange
        {
            get => (AlertRange) GetValue(AlertRangeProperty);
            set => SetValue(AlertRangeProperty, value);
        }

        private static void RangeChangeCallback(DependencyObject d, DependencyPropertyChangedEventArgs e)
        {
            var alert = (AlertRangePicker) d;
            var range = alert.AlertRange;
            if (range == null)
            {
                //sutff
            }
            else if (range is OnceAlertRange once)
            {
            }
            else if (range is RepeatingAlertRange repeat)
            {
            }
        }
    }
}