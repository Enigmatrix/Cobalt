using System;
using System.Timers;
using Cobalt.Common.Data;

namespace Cobalt.Engine
{
    public class InteractionWatcher
    {
        private readonly HookManager _hookMgr;
        private readonly Win32.HookProc _keyboardCallback;
        private readonly Win32.HookProc _mouseCallback;
        private readonly Timer _timer;
        private bool _interaction;

        public InteractionWatcher(HookManager hookMgr)
        {
            _hookMgr = hookMgr;
            _keyboardCallback = KeyboardCallback;
            _mouseCallback = MouseCallback;
            _hookMgr.WindowsHook(Win32.HookType.WH_KEYBOARD_LL, _keyboardCallback);
            _hookMgr.WindowsHook(Win32.HookType.WH_MOUSE_LL, _mouseCallback);
            _timer = new Timer(1000) {AutoReset = true};
            _timer.Elapsed += TimerCallback;
            _interaction = false;
            _timer.Start();
        }

        private void TimerCallback(object state, ElapsedEventArgs e)
        {
            if (!_interaction) return;
            _interaction = false;
            if (IdleObtained != null)
                IdleObtained(this, new InteractionEventArgs(new Interaction {Timestamp = e.SignalTime}));
        }

        private IntPtr MouseCallback(int code, IntPtr wparam, IntPtr lparam)
        {
            if (code >= 0)
                IdleCallback();
            return Win32.CallNextHookEx((int) Win32.HookType.WH_MOUSE_LL, code, wparam, lparam);
        }

        private IntPtr KeyboardCallback(int code, IntPtr wparam, IntPtr lparam)
        {
            if (code >= 0)
                IdleCallback();
            return Win32.CallNextHookEx((int) Win32.HookType.WH_KEYBOARD_LL, code, wparam, lparam);
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