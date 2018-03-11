using System;
using System.Runtime.InteropServices;
using Microsoft.Win32;
using Serilog;

namespace Cobalt.Engine
{
    public enum SystemStateChange
    {
        Shutdown,
        Logoff,
        MonitorOn,
        MonitorOff,
        Suspend,
        Resume
    }

    public class SystemWatcher
    {
        private bool _locked;
        private bool _prevMonitorOn = true;

        public SystemWatcher(MessageWindow window)
        {
            //TODO try to get session end, failing
            Win32.SetConsoleCtrlHandler(AppSessionEnded, true);
            AppDomain.CurrentDomain.ProcessExit += (sender, args) => AppSessionEnded(Win32.CtrlType.CTRL_BREAK_EVENT);
            SystemEvents.SessionEnded += SessionEnded;
            SystemEvents.SessionSwitch += SessionSwitch;

            Win32.RegisterPowerSettingNotification(window.WindowHandle,
                ref Win32.GUID_CONSOLE_DISPLAY_STATE,
                Win32.DEVICE_NOTIFY_WINDOW_HANDLE);

            window.AddHook(Win32.WindowMessages.POWERBROADCAST, (hwnd, msg, wparam, lparam) =>
            {
                if (wparam.ToInt32() != Win32.PBT_POWERSETTINGCHANGE) return;
                var bmsg = Marshal.PtrToStructure<Win32.POWERBROADCAST_SETTING>(lparam);
                if (bmsg.Data == 2) return;
                var monitorOn = bmsg.Data == 1;
                if (_prevMonitorOn == monitorOn) return;

                Log.Information("Monitor State Changed: {reason}", monitorOn ? "on" : "off");
                if (!_locked)
                    RaiseSystemMainStateChanged(monitorOn ? SystemStateChange.MonitorOn : SystemStateChange.MonitorOff);
                _prevMonitorOn = monitorOn;
            });

            Log.Information("Session SessionStart!");
        }

        private void SessionSwitch(object sender, SessionSwitchEventArgs e)
        {
            switch (e.Reason)
            {
                case SessionSwitchReason.SessionLock:
                    RaiseSystemMainStateChanged(SystemStateChange.Suspend);
                    _locked = true;
                    break;
                case SessionSwitchReason.SessionUnlock:
                    RaiseSystemMainStateChanged(SystemStateChange.Resume);
                    _locked = false;
                    break;
            }
        }


        public event EventHandler<SystemStateChangedArgs> SystemMainStateChanged = delegate { };

        //TODO doesnt work!
        private bool AppSessionEnded(Win32.CtrlType ctrlType)
        {
            Log.Information("Session Ended!");
            return true;
        }

        private void RaiseSystemMainStateChanged(SystemStateChange stateChange)
        {
            SystemMainStateChanged(this, new SystemStateChangedArgs(stateChange));
        }

        private void SessionEnded(object sender, SessionEndedEventArgs e)
        {
            Log.Information("Session Ending, Reason: {reason}", e.Reason);
            if (e.Reason == SessionEndReasons.Logoff)
                RaiseSystemMainStateChanged(SystemStateChange.Logoff);
            else if (e.Reason == SessionEndReasons.SystemShutdown)
                RaiseSystemMainStateChanged(SystemStateChange.Shutdown);
        }
    }

    public class SystemStateChangedArgs : EventArgs
    {
        public SystemStateChangedArgs(SystemStateChange newState)
        {
            ChangedToState = newState;
        }

        public SystemStateChange ChangedToState { get; }
    }
}