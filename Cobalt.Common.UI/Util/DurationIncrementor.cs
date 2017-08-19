using System;
using Cobalt.Common.UI.ViewModels;
using Cobalt.Common.Util;

namespace Cobalt.Common.UI.Util
{
    public interface IDurationIncrementor : IDisposable
    {
        void Release();
        void Increment(IHasDuration appDur);
    }

    public class DurationIncrementor : IDurationIncrementor
    {
        private readonly object _sync = new object();
        private TimeSpan _ticked;

        public DurationIncrementor(IGlobalClock clock)
        {
            Clock = clock;
            Clock.OnTick(Tick);
        }

        private IGlobalClock Clock { get; }
        private IHasDuration DurationHolder { get; set; }

        public void Increment(IHasDuration appDur)
        {
            lock (_sync)
            {
                DurationHolder = appDur;
            }
        }


        public void Release()
        {
            lock (_sync)
            {
                if (DurationHolder == null) return;
                DurationHolder.Duration -= _ticked;
                _ticked = TimeSpan.Zero;
                DurationHolder = null;
            }
        }

        public void Dispose()
        {
            Clock.ReleaseTick(Tick);
        }

        private void Tick(TimeSpan t)
        {
            lock (_sync)
            {
                if (DurationHolder == null) return;
                _ticked += t;
                DurationHolder.Duration += t;
            }
        }
    }
}