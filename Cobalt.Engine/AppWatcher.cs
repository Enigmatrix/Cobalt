using System;
using System.Reactive.Subjects;
using Cobalt.Common.Data.Entities;
using Serilog;
using static Cobalt.Engine.Win32;

namespace Cobalt.Engine
{
    public class AppWatcher
    {
        private readonly object _appUsageLock = new object();
        private readonly WinEventDelegate _foregroundWindowChangedCallback;
        private readonly AppInfoResolver _resolver;
        private readonly Subject<(AppUsage, App)> _switches;

        private AppUsageEndReason _endReason;
        private AppUsage _prev;
        private DateTime _prevFgChangeTime;
        private string _prevPath;
        private bool _recording;
        private AppUsageStartReason _startReason = AppUsageStartReason.Start;

        public AppWatcher(AppInfoResolver resolver)
        {
            _resolver = resolver;
            _prevPath = GetForegroundWindowPath();
            _prevFgChangeTime = DateTime.Now;
            _recording = true;
            _foregroundWindowChangedCallback = ForegroundWindowChangedCallback;
            _switches = new Subject<(AppUsage, App)>();
        }

        public IObservable<(AppUsage Previous, App Active)> Switches => _switches;

        public void Start()
        {
            var hook = SetWinEventHook(WinEvent.SYSTEM_FOREGROUND, WinEvent.SYSTEM_FOREGROUND,
                IntPtr.Zero, _foregroundWindowChangedCallback, 0, 0, HookFlags.OUTOFCONTEXT).Handled();
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
            var hwnd = GetTopWindow(IntPtr.Zero);
            string path;
            while ((path = _resolver.WindowPath(hwnd)) == null)
                hwnd = GetWindow(hwnd, GetWindowType.GW_HWNDNEXT);
            return path;
        }

        public AppUsage ForegroundAppUsage(DateTime start, DateTime end, string appPath)
        {
            return new AppUsage
            {
                Start = start,
                End = end,
                App = new App {Path = appPath},
                StartReason = _startReason,
                EndReason = _endReason
            };
        }

        private void RecordForegroundAppUsage(string path, DateTime endTime)
        {
            //Sometimes the duration is negative, but only by a few nanoseconds.
            //It is mostly harmless, but the charts considers them as having larger duration than other durations
            if (!_recording || endTime < _prevFgChangeTime) return;
            _prev = ForegroundAppUsage(_prevFgChangeTime, endTime, _prevPath);
            _switches.OnNext((_prev, path == null ? null : new App {Path = path}));
            _prevFgChangeTime = endTime;
            _prevPath = path;

            //reset
            _startReason = AppUsageStartReason.Switch;
            _endReason = AppUsageEndReason.Switch;
        }


        private void ForegroundWindowChangedCallback(
            IntPtr hwineventhook, WinEvent eventtype, IntPtr hwnd, int idobject, int idchild, uint dweventthread,
            uint dwmseventtime)
        {
            Log.Information("Switch received");
            var dwmsTimestamp = DateTime.Now.AddMilliseconds(dwmseventtime - Environment.TickCount);

            lock (_appUsageLock)
            {
                var path = _resolver.WindowPath(hwnd);
                if (string.IsNullOrEmpty(path) || path == _prevPath) return;

                RecordForegroundAppUsage(path, dwmsTimestamp);
            }
        }
    }
}