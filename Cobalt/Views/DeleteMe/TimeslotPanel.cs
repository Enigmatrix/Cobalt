using System;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;
using Cobalt.Common.UI.ViewModels;

namespace Cobalt.Views.DeleteMe
{
    public class TimeslotPanel : Panel
    {
        public static readonly DependencyProperty StartProperty = DependencyProperty.RegisterAttached("Start",
            typeof(DateTime), typeof(TimeslotPanel),
            new FrameworkPropertyMetadata {AffectsArrange = true, AffectsMeasure = true});

        public static readonly DependencyProperty EndProperty = DependencyProperty.RegisterAttached("End",
            typeof(DateTime), typeof(TimeslotPanel),
            new FrameworkPropertyMetadata {AffectsArrange = true, AffectsMeasure = true});

        public static void SetStart(UIElement element,
            DateTime value)
        {
            element.SetValue(StartProperty, value);
        }

        public static DateTime GetStart(UIElement element)
        {
            return (DateTime) element.GetValue(StartProperty);
        }

        public static void SetEnd(UIElement element,
            DateTime value)
        {
            element.SetValue(EndProperty, value);
        }

        public static DateTime GetEnd(UIElement element)
        {
            return (DateTime) element.GetValue(EndProperty);
        }

        protected override Size MeasureOverride(Size availableSize)
        {
            return base.MeasureOverride(availableSize);
        }

        protected override Size ArrangeOverride(Size finalSize)
        {
            var height = finalSize.Height;
            var width = finalSize.Width;
            var start = 0.0;
            var arrangeHeight = 0.0;
            foreach (UIElement element in InternalChildren)
            {
                var e = (ListBoxItem) element;
                var g = ((IAppUsageViewModel) e.Content);
                start = DateTimeDayScale(g.StartTimestamp, height);
                arrangeHeight = DateTimeDayScale(g.EndTimestamp, height) - DateTimeDayScale(g.StartTimestamp, height);
                start = Math.Max(0, start);
                arrangeHeight = Math.Max(0, arrangeHeight);
                element.Arrange(new Rect(0,start,width,arrangeHeight));

            }
            return finalSize;
        }

        private double DateTimeDayScale(DateTime time, double height)
        {
            return ((double)(time-DateTime.Today.AddDays(-2)).Ticks/TimeSpan.TicksPerDay)*height;
        }
    }
}