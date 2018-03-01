using Cobalt.Common.Data;
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

namespace Cobalt.Views.Controls
{
    public partial class AlertRangePickerDialog
    {
        public AlertRangePickerDialog()
        {
            InitializeComponent();
            Loaded += (o,e) =>
            {
                Type.SelectionChanged += ComboBox_Selected;
                Type.SelectedItem = Once;
                var dataContent = this.DataContext;
            };
        }
        public AlertRange AlertRange
        {
            get { return (AlertRange)GetValue(AlertRangeProperty); }
            set { SetValue(AlertRangeProperty, value); }
        }

        public static readonly DependencyProperty AlertRangeProperty =
            DependencyProperty.Register("AlertRange", typeof(AlertRange), typeof(AlertRangePickerDialog), new PropertyMetadata(null, RangeChangeCallback));

        private static void RangeChangeCallback(DependencyObject d, DependencyPropertyChangedEventArgs e)
        {
            var alert = (AlertRangePickerDialog) d;
            var range = alert.AlertRange;
            if(range == null)
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
            OnceRangeDetail.Visibility = ReferenceEquals(Type.SelectedItem, Once) ? Visibility.Visible : Visibility.Collapsed;
            RepeatRangeDetail.Visibility = ReferenceEquals(Type.SelectedItem, Once) ? Visibility.Collapsed : Visibility.Visible;
            if(ReferenceEquals(Type.SelectedItem, Once))
            {
                Frequency.Text="";
            }
            else if (ReferenceEquals(Type.SelectedItem, Daily))
            {

                Frequency.Text="per day";
            }
            else if (ReferenceEquals(Type.SelectedItem, Weekly))
            {

                Frequency.Text="daily per week";
            }
            else if (ReferenceEquals(Type.SelectedItem, Monthly))
            {

                Frequency.Text="daily per month";
            }
        }
    }
}
