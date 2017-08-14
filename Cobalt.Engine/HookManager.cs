using System;
using System.ComponentModel;
using System.Runtime.InteropServices;

namespace Cobalt.Engine
{
    public class HookManager
    {
        /// <summary>
        ///     Call this after setting up hooks
        /// </summary>
        public void EventLoop()
        {
            //keep getting messages
            while (true)
            {
                if (Win32.GetMessage(out Win32.MSG msg, IntPtr.Zero, 0, 0) == 0) break;

                //even ms docs say that you dont need to understand these
                Win32.TranslateMessage(ref msg);
                Win32.DispatchMessage(ref msg);
            }
        }

        public void Hook(Win32.WinEvent @event, Win32.WinEventProc callback)
        {
            HookRange(@event, @event, callback);
        }

        public void HookRange(Win32.WinEvent min, Win32.WinEvent max, Win32.WinEventProc callback)
        {
            var windowEventHook = Win32.SetWinEventHook(
                (int)min, // eventMin
                (int)max, // eventMax
                IntPtr.Zero, // hmodWinEventProc
                callback, // lpfnWinEventProc
                0, // idProcess 
                0, // idThread 
                //since both of the above params are 0, its  a global hook
                Win32.WINEVENT_OUTOFCONTEXT);

            if (windowEventHook == IntPtr.Zero)
                throw new Win32Exception(Marshal.GetLastWin32Error());
        }
    }
}