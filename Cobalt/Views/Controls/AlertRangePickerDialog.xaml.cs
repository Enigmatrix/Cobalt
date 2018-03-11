using System.Windows;
using Cobalt.Common.Data;

namespace Cobalt.Views.Controls
{
    public partial class AlertRangePickerDialog
    {
        public static readonly DependencyProperty AlertRangeProperty =
            DependencyProperty.Register("AlertRange", typeof(AlertRange), typeof(AlertRangePickerDialog),
                new PropertyMetadata(null, RangeChangeCallback));

        public AlertRangePickerDialog()
        {
            InitializeComponent();
            Loaded += (o, e) =>
            {
                Type.SelectionChanged += ComboBox_Selected;
                Type.SelectedItem = Once;
                var dataContent = DataContext;
            };
        }

        public AlertRange AlertRange
        {
            get => (AlertRange) GetValue(AlertRangeProperty);
            set => SetValue(AlertRangeProperty, value);
        }

        private static void RangeChangeCallback(DependencyObject d, DependencyPropertyChangedEventArgs e)
        {
            var alert = (AlertRangePickerDialog) d;
            var range = alert.AlertRange;
            if (range == null)
            {
                //stuff
            }
            else if (range is OnceAlertRange once)
            {
            }
            else if (range is RepeatingAlertRange repeat)
            {
            }
        }

        private void ComboBox_Selected(object sender, RoutedEventArgs e)
        {
            OnceRangeDetail.Visibility =
                ReferenceEquals(Type.SelectedItem, Once) ? Visibility.Visible : Visibility.Collapsed;
            RepeatRangeDetail.Visibility =
                ReferenceEquals(Type.SelectedItem, Once) ? Visibility.Collapsed : Visibility.Visible;
            if (ReferenceEquals(Type.SelectedItem, Once))
                Frequency.Text = "";
            else if (ReferenceEquals(Type.SelectedItem, Daily))
                Frequency.Text = "per day";
            else if (ReferenceEquals(Type.SelectedItem, Weekly))
                Frequency.Text = "daily per week";
            else if (ReferenceEquals(Type.SelectedItem, Monthly))
                Frequency.Text = "daily per month";
        }
    }
}