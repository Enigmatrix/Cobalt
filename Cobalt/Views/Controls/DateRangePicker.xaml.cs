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
        public static readonly DependencyProperty StartDateSetProperty =
            DependencyProperty.RegisterAttached("StartDateSet", typeof(DateTime?), typeof(DateRangePicker),
                new PropertyMetadata(DateTime.Today.Subtract(TimeSpan.FromDays(1)), CustomStartDateChanged));

        public static readonly DependencyProperty EndDateSetProperty =
            DependencyProperty.RegisterAttached("EndDateSet", typeof(DateTime?), typeof(DateRangePicker),
                new PropertyMetadata(DateTime.Today, CustomEndDateChanged));

        public static readonly DependencyProperty StartTimeSetProperty =
            DependencyProperty.RegisterAttached("StartTimeSet", typeof(DateTime?), typeof(DateRangePicker),
                new PropertyMetadata(DateTime.Today.Subtract(TimeSpan.FromDays(1)), CustomStartTimeChanged));

        public static readonly DependencyProperty EndTimeSetProperty =
            DependencyProperty.RegisterAttached("EndTimeSet", typeof(DateTime?), typeof(DateRangePicker),
                new PropertyMetadata(DateTime.Today, CustomEndTimeChanged));


        public static readonly DependencyProperty StartProperty =
            DependencyProperty.Register("Start", typeof(DateTime?), typeof(DateRangePicker),
                new FrameworkPropertyMetadata(null));

        public static readonly DependencyProperty EndProperty =
            DependencyProperty.Register("End", typeof(DateTime?), typeof(DateRangePicker),
                new FrameworkPropertyMetadata(null));

        public DateRangePicker()
        {
            InitializeComponent();
            Loaded += (s, o) =>
            {
                SelectionChanged += DateRangePicker_OnSelected;
                SelectedIndex = 0;
            };
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

        public static bool GetStartDateSet(DependencyObject obj)
        {
            return (bool) obj.GetValue(StartDateSetProperty);
        }

        public static void SetStartDateSet(DependencyObject obj, DateTime? value)
        {
            obj.SetValue(StartDateSetProperty, value);
        }

        public static bool GetEndDateSet(DependencyObject obj)
        {
            return (bool) obj.GetValue(EndDateSetProperty);
        }

        public static void SetEndDateSet(DependencyObject obj, DateTime? value)
        {
            obj.SetValue(EndDateSetProperty, value);
        }

        public static bool GetStartTimeSet(DependencyObject obj)
        {
            return (bool) obj.GetValue(StartTimeSetProperty);
        }

        public static void SetStartTimeSet(DependencyObject obj, DateTime? value)
        {
            obj.SetValue(StartTimeSetProperty, value);
        }

        public static bool GetEndTimeSet(DependencyObject obj)
        {
            return (bool) obj.GetValue(EndTimeSetProperty);
        }

        public static void SetEndTimeSet(DependencyObject obj, DateTime? value)
        {
            obj.SetValue(EndTimeSetProperty, value);
        }

        private static DateRangePicker GetRangePicker(DependencyObject d)
        {
            while (d != null && d.GetType() != typeof(DateRangePicker))
                d = d.GetParentObject();
            return (DateRangePicker) d;
        }

        private static void CustomStartDateChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
        {
            var dis = GetRangePicker(d);
            if (e.NewValue == null)
            {
                dis.Start = null;
                return;
            }

            var ne = (DateTime) e.NewValue;
            var start = dis.Start;
            dis.Start = start?.Add(ne - start.Value.Date) ?? ne;
        }

        private static void CustomEndDateChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
        {
            var dis = GetRangePicker(d);
            if (e.NewValue == null)
            {
                dis.Start = null;
                return;
            }

            var ne = (DateTime) e.NewValue;
            var end = dis.End;
            dis.End = end?.Add(ne - end.Value.Date) ?? ne;
        }

        private static void CustomStartTimeChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
        {
            var dis = GetRangePicker(d);
            if (e.NewValue == null)
            {
                dis.Start = null;
                return;
            }

            var ne = (DateTime) e.NewValue;
            var start = dis.Start;
            dis.Start = start?.Add(ne.TimeOfDay - start.Value.TimeOfDay) ?? ne;
        }

        private static void CustomEndTimeChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
        {
            var dis = GetRangePicker(d);
            if (e.NewValue == null)
            {
                dis.Start = null;
                return;
            }

            var ne = (DateTime) e.NewValue;
            var end = dis.End;
            dis.End = end?.Add(ne.TimeOfDay - end.Value.TimeOfDay) ?? ne;
        }

        private void DateRangePicker_OnSelected(object sender, RoutedEventArgs e)
        {
            var today = DateTime.Today;
            if (ReferenceEquals(SelectedItem, Today))
                (Start, End) = (today, null);
            else if (ReferenceEquals(SelectedItem, Yesterday))
                (Start, End) = (today.Subtract(TimeSpan.FromDays(1)), today);
            else if (ReferenceEquals(SelectedItem, ThisWeek))
                (Start, End) = (today.AddDays(-(int) today.DayOfWeek), null);
            else if (ReferenceEquals(SelectedItem, ThisMonth))
                (Start, End) = (today.AddDays(1 - today.Day), null);
            else
                (Start, End) = (null, null);
        }
    }
}