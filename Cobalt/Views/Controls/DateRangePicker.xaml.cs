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
    /// Interaction logic for DateRangePicker.xaml
    /// </summary>
    public partial class DateRangePicker
    {
        public DateRangePicker()
        {
            InitializeComponent();
        }



        public DateTime Start
        {
            get => (DateTime)GetValue(StartProperty);
            set => SetValue(StartProperty, value);
        }

        public static readonly DependencyProperty StartProperty =
            DependencyProperty.Register("Start", typeof(DateTime), typeof(DateRangePicker), new PropertyMetadata(DateTime.Today.Subtract(TimeSpan.FromDays(1))));


        public DateTime End
        {
            get => (DateTime)GetValue(EndProperty);
            set => SetValue(EndProperty, value);
        }

        public static readonly DependencyProperty EndProperty =
            DependencyProperty.Register("End", typeof(DateTime), typeof(DateRangePicker), new PropertyMetadata(DateTime.Today));


    }
}
