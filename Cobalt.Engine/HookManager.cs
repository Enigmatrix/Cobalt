using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using Cobalt.Engine.Util;

namespace Cobalt.Engine
{
    public class HookManager
    {
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