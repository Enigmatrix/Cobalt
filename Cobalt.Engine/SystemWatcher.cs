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
        Suspend,
        Resume,
        MonitorOn,
        MonitorOff
    }

    public class SystemWatcher
    {
        private bool _prevMonitorOn = true;

        public SystemWatcher(MessageWindow window)
        {
            //TODO try to get session end, failing
            Win32.SetConsoleCtrlHandler(AppSessionEnded, true);
            AppDomain.CurrentDomain.ProcessExit += (sender, args) => AppSessionEnded(Win32.CtrlType.CTRL_BREAK_EVENT);
            SystemEvents.PowerModeChanged += PowerModeChanged;
            SystemEvents.SessionEnded += SessionEnded;

            Win32.RegisterPowerSettingNotification(window.WindowHandle,
                ref Win32.GUID_MONITOR_POWER_ON,
                Win32.DEVICE_NOTIFY_WINDOW_HANDLE);

            window.AddHook(Win32.WindowMessages.POWERBROADCAST, (hwnd, msg, wparam, lparam) =>
            {
                if (wparam.ToInt32() != Win32.PBT_POWERSETTINGCHANGE) return;
                var bmsg = Marshal.PtrToStructure<Win32.POWERBROADCAST_SETTING>(lparam);
                var monitorOn = bmsg.Data == 1;
                if (_prevMonitorOn == monitorOn) return;

                RaiseSystemMainStateChanged(monitorOn ? SystemStateChange.MonitorOn : SystemStateChange.MonitorOff);
                _prevMonitorOn = monitorOn;
            });

            Log.Information("Session SessionStart!");
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

        private void PowerModeChanged(object sender, PowerModeChangedEventArgs e)
        {
            //TODO buggy: Sleep is called after monitoroff when device is going to sleep
            /*
            if (e.Mode == PowerModes.StatusChange) return;
            Log.Information("{stateChange} Session", e.Mode);
            if (e.Mode == PowerModes.Suspend)
                RaiseSystemMainStateChanged(SystemStateChange.Suspend);
            else if (e.Mode == PowerModes.Resume)
                RaiseSystemMainStateChanged(SystemStateChange.Resume);
                */
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