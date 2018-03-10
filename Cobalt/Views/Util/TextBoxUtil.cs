using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Input;

namespace Cobalt.Views.Util
{
    public class TextBoxUtil
    {
        public static readonly RoutedEvent EnterUpEvent = EventManager.RegisterRoutedEvent("EnterUp", RoutingStrategy.Bubble,
            typeof(RoutedEventHandler), typeof(TextBoxUtil));

        public static void AddEnterUpHandler(DependencyObject d, RoutedEventHandler h)
        {
            if (!(d is UIElement elem)) return;
            elem.KeyUp += HandleKeyUp;
            elem.AddHandler(EnterUpEvent, h);
        }

        private static void HandleKeyUp(object sender, KeyEventArgs e)
        {
            if(e.Key == Key.Enter)
                ((UIElement)sender).RaiseEvent(new RoutedEventArgs(EnterUpEvent));
        }

        public static void RemoveEnterUpHandler(DependencyObject d, RoutedEventHandler h)
        {
            if (!(d is UIElement elem)) return;
            elem.KeyUp -= HandleKeyUp;
            elem.RemoveHandler(EnterUpEvent, h);
        }


    }
}
