using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Runtime.InteropServices;

namespace Cobalt.Engine
{
    public class MessageWindow
    {
        private const string WndClass = "Cobalt_InvisWin";

        private static readonly Lazy<MessageWindow> _instance =
            new Lazy<MessageWindow>(() => new MessageWindow());

        private readonly Win32.WndProc _wndProc;

        private readonly Dictionary<Win32.WindowMessages,
            Action<IntPtr, Win32.WindowMessages, IntPtr, IntPtr>> handlers;

        public MessageWindow()
        {
            _wndProc = WndProc;
            handlers = new Dictionary<Win32.WindowMessages, Action<IntPtr, Win32.WindowMessages, IntPtr, IntPtr>>();
            var wnd = new Win32.WNDCLASSEX
            {
                cbSize = (uint)Marshal.SizeOf<Win32.WNDCLASSEX>(),
                hInstance = Process.GetCurrentProcess().Handle,
                lpszClassName = WndClass,
                lpfnWndProc = _wndProc
            };
            Win32.RegisterClassEx(ref wnd);

            WindowHandle = Win32.CreateWindowEx(Win32.WindowStylesEx.WS_EX_LEFT, WndClass, WndClass,
                Win32.WindowStyles.WS_OVERLAPPED, 0, 0, 0, 0, Win32.HWND_MESSAGE, IntPtr.Zero, IntPtr.Zero, IntPtr.Zero);
        }

        public IntPtr WindowHandle { get; }

        public static MessageWindow Instance => _instance.Value;

        //TODO allow hooking mechanism thru other methods
        private IntPtr WndProc(IntPtr hwnd, Win32.WindowMessages msg, IntPtr wparam, IntPtr lparam)
        {
            if (handlers.ContainsKey(msg))
                handlers[msg](hwnd, msg, wparam, lparam);
            return Win32.DefWindowProc(hwnd, msg, wparam, lparam);
        }

        public void AddHook(Win32.WindowMessages msg,
            Action<IntPtr, Win32.WindowMessages, IntPtr, IntPtr> handler)
        {
            handlers[msg] = handler;
        }
    }
}