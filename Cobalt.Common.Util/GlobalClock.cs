using System;
using System.Runtime.InteropServices;

namespace Cobalt.Common.Util
{
    public interface IGlobalClock
    {
        void OnTick(Action<TimeSpan> tick);
        void ReleaseTick(Action<TimeSpan> tick);
    }

    /// <summary>
    ///     This timer class uses unmanaged DLL for better accuracy at short frequencies.
    ///     This class is not thread safe.
    ///     http://stackoverflow.com/questions/416522/c-sharp-why-are-timer-frequencies-extremely-off
    /// </summary>
    public class GlobalClock : IGlobalClock
    {
        private const string WINMM = "winmm.dll";
        private readonly MMTimerProc _callbackFunction;
        private TimeSpan _interval;
        private uint _timerId;

        public GlobalClock(TimeSpan t)
        {
            _callbackFunction = CallbackFunction;
            _interval = t;
            Start();
        }

        public virtual void Dispose()
        {
            Stop();
        }

        public virtual void Start()
        {
            StartUnmanagedTimer();

            if (_timerId == 0)
                throw new Exception("TimeSet Event Error");
        }

        public void Stop()
        {
            if (_timerId == 0) return;
            StopUnmanagedTimer();
        }

        public void UpdateTimeInterval(TimeSpan interval)
        {
            Stop();
            this._interval = interval;
            Start();
        }

        private void StartUnmanagedTimer()
        {
            _timerId = timeSetEvent((uint)_interval.TotalMilliseconds, 0, _callbackFunction, 0, 1);
        }

        private void StopUnmanagedTimer()
        {
            timeKillEvent(_timerId);
            _timerId = 0;
        }

        private void CallbackFunction(uint timerid, uint msg, IntPtr user, uint dw1, uint dw2)
        {
            Callback?.Invoke(_interval);
        }

        public event Action<TimeSpan> Callback;

        [DllImport(WINMM)]
        private static extern uint timeSetEvent(
            uint uDelay,
            uint uResolution,
            [MarshalAs(UnmanagedType.FunctionPtr)] MMTimerProc lpTimeProc,
            uint dwUser,
            int fuEvent
        );

        [DllImport(WINMM)]
        private static extern uint timeKillEvent(uint uTimerID);

        #region Nested type: MMTimerProc

        private delegate void MMTimerProc(uint timerid, uint msg, IntPtr user, uint dw1, uint dw2);

        #endregion

        public void OnTick(Action<TimeSpan> tick)
        {
            Callback += tick;
        }

        public void ReleaseTick(Action<TimeSpan> tick)
        {
            Callback -= tick;
        }
    }
}
