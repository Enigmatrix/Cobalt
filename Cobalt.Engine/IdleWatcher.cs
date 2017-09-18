using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Cobalt.Engine
{
    public class IdleWatcher
    {
        private HookManager _hookMgr;
        private Win32.HookProc _keyboardCallback, _mouseCallback;

        public IdleWatcher(HookManager hookMgr)
        {
            _hookMgr = hookMgr;
            _keyboardCallback = KeyboardCallback;
            _mouseCallback = MouseCallback;
            _hookMgr.WindowsHook(Win32.HookType.WH_KEYBOARD_LL, _keyboardCallback);
            _hookMgr.WindowsHook(Win32.HookType.WH_MOUSE_LL, _mouseCallback);
        }

        private IntPtr MouseCallback(int code, IntPtr wparam, IntPtr lparam)
        {
            IdleCallback();
            return Win32.CallNextHookEx((int)Win32.HookType.WH_MOUSE_LL, code, wparam, lparam);
        }

        private IntPtr KeyboardCallback(int code, IntPtr wparam, IntPtr lparam)
        {
            IdleCallback();
            return Win32.CallNextHookEx((int)Win32.HookType.WH_KEYBOARD_LL, code, wparam, lparam);
        }

        private void IdleCallback()
        {

        }

        public event EventHandler<IdleEventArgs> IdleObtained;
    }

    public class IdleEventArgs : EventArgs
    {
        
    }
}
