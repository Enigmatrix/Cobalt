using System;
using Cobalt.Common.UI.ViewModels;
using Cobalt.Common.Util;

namespace Cobalt.Common.UI.Util
{
    public interface IDurationIncrementor : IDisposable
    {
        void Release();
        void Increment(IAppDurationViewModel appDur);
    }

    public class DurationIncrementor : IDurationIncrementor
    {
        private TimeSpan _ticked;

        public DurationIncrementor(IGlobalClock clock)
        {
            Clock = clock;
            Clock.OnTick(Tick);
        }

        private IGlobalClock Clock { get; }
        private IAppDurationViewModel AppDuration { get; set; }

        private readonly object _sync = new object();

        public void Increment(IAppDurationViewModel appDur)
        {
            lock (_sync)
            {
                AppDuration = appDur;
            }
        }


        public void Release()
        {
            lock (_sync)
            {
                if(AppDuration == null) return;
                AppDuration.Duration -= _ticked;
                _ticked = TimeSpan.Zero;
                AppDuration = null;
            }
        }

        private void Tick(TimeSpan t)
        {
            lock (_sync)
            {
                if(AppDuration == null) return;
                _ticked += t;
                AppDuration.Duration += t;
            }
        }

        public void Dispose()
        {
            Clock.ReleaseTick(Tick);
        }
    }
}