using System;
using System.Threading;
using Cobalt.Common.Data;

namespace Cobalt.Engine
{
    public class InteractionWatcher
    {
        private HookManager _hookMgr;
        private Win32.HookProc _keyboardCallback, _mouseCallback;
        private Timer _timer;
        private bool _interaction;

        public InteractionWatcher(HookManager hookMgr)
        {
            _hookMgr = hookMgr;
            _keyboardCallback = KeyboardCallback;
            _mouseCallback = MouseCallback;
            _hookMgr.WindowsHook(Win32.HookType.WH_KEYBOARD_LL, _keyboardCallback);
            _hookMgr.WindowsHook(Win32.HookType.WH_MOUSE_LL, _mouseCallback);
            _timer = new Timer(TimerCallback, null, 0, 1000);
            _interaction = false;
        }

        private void TimerCallback(object state)
        {
            if (!_interaction) return;
            _interaction = false;
            if(IdleObtained != null)
                IdleObtained(this, new InteractionEventArgs(new Interaction{Timestamp = DateTime.Now.AddSeconds(-1)}));
        }

        private IntPtr MouseCallback(int code, IntPtr wparam, IntPtr lparam)
        {
            if (code >= 0)
                IdleCallback();
            return Win32.CallNextHookEx((int)Win32.HookType.WH_MOUSE_LL, code, wparam, lparam);
        }

        private IntPtr KeyboardCallback(int code, IntPtr wparam, IntPtr lparam)
        {
            if (code >= 0)
                IdleCallback();
            return Win32.CallNextHookEx((int)Win32.HookType.WH_KEYBOARD_LL, code, wparam, lparam);
        }


        private void IdleCallback()
        {
            _interaction = true;
        }

        public event EventHandler<InteractionEventArgs> IdleObtained;
    }

    public class InteractionEventArgs : EventArgs
    {
        public InteractionEventArgs(Interaction interaction)
        {
            Interaction = interaction;
        }

        public Interaction Interaction { get; set; }
    }
}
