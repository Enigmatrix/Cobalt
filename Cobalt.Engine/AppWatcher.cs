using System;
using Cobalt.Common.Data;

namespace Cobalt.Engine
{
    public class AppWatcher
    {
        private readonly object _appUsageLock = new object();
        private readonly Win32.WinEventProc _foregroundWindowChangedCallback;
        private readonly HookManager _hookMgr;

        private readonly PathResolver _pathResolver;
        private AppUsageEndReason _endReason;
        private AppUsage _prev;
        private DateTime _prevFgChangeTime;
        private string _prevPath;
        private bool _recording;
        private AppUsageStartReason _startReason = AppUsageStartReason.Start;

        public AppWatcher(HookManager hookMgr)
        {
            _pathResolver = new PathResolver();
            _hookMgr = hookMgr;
            _foregroundWindowChangedCallback = ForegroundWindowChangedCallback;

            _prevPath = GetForegroundWindowPath();
            _prevFgChangeTime = DateTime.Now;
            _recording = true;

            _hookMgr.WinEventHook(Win32.WinEvent.EVENT_SYSTEM_FOREGROUND, _foregroundWindowChangedCallback);
        }

        //TODO better naming
        public event EventHandler<ForegroundAppSwitchEventArgs> ForegroundAppUsageObtained = delegate { };


        public void EventLoop()
        {
            _hookMgr.EventLoop();
        }

        private void RecordForegroundAppUsage(string path, DateTime endTime)
        {
            //Sometimes the duration is negative, but only by a few nanoseconds.
            //It is mostly harmless, but the charts considers them as having larger duration than other durations
            if (!_recording || endTime < _prevFgChangeTime) return;
            _prev = ForegroundAppUsage(_prevFgChangeTime, endTime, _prevPath);
            ForegroundAppUsageObtained(this,
                new ForegroundAppSwitchEventArgs(_prev, path == null ? null : new App {Path = path}));
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
            var dwmsTimestamp = DateTime.Now.AddMilliseconds(dwmseventtime - Environment.TickCount);
            lock (_appUsageLock)
            {
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
                if (!_recording) return;
                _endReason = reason;
                RecordForegroundAppUsage(null, DateTime.Now);
                _recording = false;
            }
        }

        private string GetForegroundWindowPath()
        {
            var hwnd = Win32.GetTopWindow(IntPtr.Zero);
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
                App = new App {Path = appPath},
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

        public AppUsage PreviousAppUsage { get; }
        public App NewApp { get; }
    }
}