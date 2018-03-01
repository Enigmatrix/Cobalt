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
    /// <summary>
    /// Interaction logic for AlertRangePicker.xaml
    /// </summary>
    public partial class AlertRangePicker
    {
        public AlertRangePicker()
        {
            InitializeComponent();
        }



        public AlertRange AlertRange
        {
            get { return (AlertRange)GetValue(AlertRangeProperty); }
            set { SetValue(AlertRangeProperty, value); }
        }

        public static readonly DependencyProperty AlertRangeProperty =
            DependencyProperty.Register("AlertRange", typeof(AlertRange), typeof(AlertRangePicker), new PropertyMetadata(null, RangeChangeCallback));

        private static void RangeChangeCallback(DependencyObject d, DependencyPropertyChangedEventArgs e)
        {
            var alert = (AlertRangePicker) d;
            var range = alert.AlertRange;
            if(range == null)
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
