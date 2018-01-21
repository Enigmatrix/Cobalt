using System;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Windows;
using System.Windows.Input;
using CommonWin32.API;
using CommonWin32.Rectangles;
using CommonWin32.Windows;
using MahApps.Metro.Controls;

namespace Cobalt.Views.Util
{
    public static class WindowUtils
    {
        public static readonly DependencyProperty DragMoveProperty =
            DependencyProperty.RegisterAttached("DragMoveEvent", typeof(bool), typeof(WindowUtils),
                new PropertyMetadata(false, PropertyChangedCallback));

        [DllImport("user32.dll")]
        private static extern bool GetCursorPos(out POINT lpPoint);

        private static void PropertyChangedCallback(DependencyObject dependencyObject,
            DependencyPropertyChangedEventArgs dependencyPropertyChangedEventArgs)
        {
            //var obj = dependencyObject;
            //while (!(obj is Window))
            //{
            //    obj = LogicalTreeHelper.GetParent(obj);

            //}
            var element = (FrameworkElement) dependencyObject;

            element.MouseLeftButtonDown += (s, e) =>
            {
                //for some reason if this below line is outside this scope, the program hangs
                var window = GetElementRootWindow(element);
                if (e.ClickCount == 1)
                    window.DragMove();
                else if (e.ClickCount == 2)
                    window.WindowState = window.WindowState == WindowState.Maximized
                        ? WindowState.Normal
                        : WindowState.Maximized;


                //Use https://dragablz.net/2014/12/16/getting-windows-snap-to-play-with-wpf-borderless-windows/
                //to apply soln for unmaximizing using aero snap
            };

            element.MouseMove += (s, e) =>
            {
                var window = GetElementRootWindow(element);
                if (window.WindowState != WindowState.Maximized || e.LeftButton != MouseButtonState.Pressed) return;

                POINT cursorPos;
                GetCursorPos(out cursorPos);
                var relPos = e.GetPosition(window);

                window.Top = cursorPos.y - relPos.Y;
                window.Left = cursorPos.x - relPos.X * window.RestoreBounds.Width / window.Width;

                var lParam = (int) (uint) cursorPos.x | (cursorPos.y << 16);

                var value = (IntPtr) typeof(Window)
                    .GetProperty("CriticalHandle", BindingFlags.NonPublic | BindingFlags.Instance)
                    .GetValue(window, new object[0]);


                User32.SendMessage(value, (uint) WindowMessage.WM_LBUTTONUP, (IntPtr) NcHitTest.HTCAPTION,
                    (IntPtr) lParam);
                User32.SendMessage(value, (uint) WindowMessage.WM_SYSCOMMAND, (IntPtr) 0xF012, IntPtr.Zero);
            };
        }

        public static bool GetDragMove(DependencyObject obj)
        {
            return (bool) obj.GetValue(DragMoveProperty);
        }

        public static void SetDragMove(DependencyObject obj, bool value)
        {
            obj.SetValue(DragMoveProperty, value);
        }

        public static Window GetElementRootWindow(FrameworkElement element)
        {
            var parent = element;
            while (!(parent is Window)) parent = (FrameworkElement) parent.GetParentObject();
            return (Window) parent;
        }
    }
}