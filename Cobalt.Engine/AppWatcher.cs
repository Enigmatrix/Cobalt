using System;
using Cobalt.Common.Data;

namespace Cobalt.Engine
{
    public class AppWatcher
    {
        private readonly Win32.WinEventProc _foregroundWindowChangedCallback;
        private readonly HookManager _hookMgr;

        private readonly PathResolver _pathResolver;

        private readonly object _appUsageLock = new object();
        private AppUsageEndReason _endReason;
        private AppUsage _prev;
        private DateTime _prevFgChangeTime;
        private string _prevPath;
        private bool _recording;
        private AppUsageStartReason _startReason = AppUsageStartReason.Start;

        public AppWatcher()
        {
            _pathResolver = new PathResolver();
            _hookMgr = new HookManager();
            _foregroundWindowChangedCallback = ForegroundWindowChangedCallback;

            _prevPath = GetForegroundWindowPath();
            _prevFgChangeTime = DateTime.Now;
            _recording = true;

            _hookMgr.Hook(Win32.WinEvent.EVENT_SYSTEM_FOREGROUND, _foregroundWindowChangedCallback);
        }

        //TODO better naming
        public event EventHandler<ForegroundAppSwitchEventArgs> ForegroundAppUsageObtained = delegate { };


        public void EventLoop()
        {
            _hookMgr.EventLoop();
        }

        private void RecordForegroundAppUsage(string path, DateTime endTime)
        {
            //TODO make sure
            if (!_recording) return;
            _prev = ForegroundAppUsage(_prevFgChangeTime, endTime, _prevPath);
            ForegroundAppUsageObtained(this, new ForegroundAppSwitchEventArgs(_prev, new App { Path = path }));
            _prevFgChangeTime = endTime;
            _prevPath = path;

            //reset
            _startReason = AppUsageStartReason.Switch;
            _endReason = AppUsageEndReason.Switch;
        }

        private void ForegroundWindowChangedCallback(
            IntPtr hwineventhook, uint eventtype, IntPtr hwnd, int idobject, int idchild, uint dweventthread,
            uint dwmseventtime)
        {
            lock (_appUsageLock)
            {
                var dwmsTimestamp = DateTime.Now.AddMilliseconds(dwmseventtime - Environment.TickCount);
                var path = _pathResolver.ResolveWindowPath(hwnd);

                if (string.IsNullOrEmpty(path) || path == _prevPath) return;

                RecordForegroundAppUsage(path, dwmsTimestamp);
            }
        }

        public void StartRecordingWith(AppUsageStartReason reason)
        {
            lock (_appUsageLock)
            {
                _recording = true;
                _prevPath = GetForegroundWindowPath();
                _prevFgChangeTime = DateTime.Now;
                _startReason = reason;
            }
        }

        public void EndRecordingWith(AppUsageEndReason reason)
        {
            lock (_appUsageLock)
            {
                _endReason = reason;
                RecordForegroundAppUsage(null, DateTime.Now);
                _recording = false;
            }
        }

        private string GetForegroundWindowPath()
        {
            var hwnd = Win32.GetForegroundWindow();
            string path;
            while ((path = _pathResolver.ResolveWindowPath(hwnd)) == null)
                hwnd = Win32.GetWindow(hwnd, Win32.GW_HWNDNEXT);
            return path;
        }

        public AppUsage ForegroundAppUsage(DateTime start, DateTime end, string appPath)
        {
            return new AppUsage
            {
                StartTimestamp = start,
                EndTimestamp = end,
                App = new App { Path = appPath },
                UsageType = AppUsageType.Foreground,
                UsageStartReason = _startReason,
                UsageEndReason = _endReason
            };
        }
    }

    public class ForegroundAppSwitchEventArgs : EventArgs
    {
        public ForegroundAppSwitchEventArgs(AppUsage prev, App newApp)
        {
            PreviousAppUsage = prev;
            NewApp = newApp;
        }

        public AppUsage PreviousAppUsage { get; private set; }
        public App NewApp { get; private set; }
    }
}