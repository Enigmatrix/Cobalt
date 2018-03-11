using System;
using System.Windows;
using System.Windows.Controls;

namespace Cobalt.Views.Controls
{
    public class TimePanel : Panel
    {
        public static readonly DependencyProperty StartProperty =
            DependencyProperty.RegisterAttached(
                "Start",
                typeof(DateTime),
                typeof(TimePanel),
                new PropertyMetadata()
            );

        public static readonly DependencyProperty EndProperty =
            DependencyProperty.RegisterAttached(
                "End",
                typeof(DateTime),
                typeof(TimePanel),
                new PropertyMetadata()
            );

        public static void SetStart(UIElement element, DateTime value)
        {
            element.SetValue(StartProperty, value);
        }

        public static DateTime GetStart(UIElement element)
        {
            return (DateTime) element.GetValue(StartProperty);
        }

        public static void SetEnd(UIElement element, DateTime value)
        {
            element.SetValue(EndProperty, value);
        }

        public static DateTime GetEnd(UIElement element)
        {
            return (DateTime) element.GetValue(EndProperty);
        }

        protected override Size ArrangeOverride(Size finalSize)
        {
            return finalSize;
        }

        protected override Size MeasureOverride(Size availableSize)
        {
            var h = availableSize.Height;
            var w = availableSize.Width;
            foreach (UIElement child in InternalChildren)
            {
                var ts = (GetStart(child) - DateTime.Today).Ticks * w / TimeSpan.TicksPerDay;
                var te = (GetEnd(child) - DateTime.Today).Ticks * w / TimeSpan.TicksPerDay;

                child.Arrange(new Rect(new Point(ts, 0), new Point(te, h)));
            }

            return availableSize;
        }
    }
}