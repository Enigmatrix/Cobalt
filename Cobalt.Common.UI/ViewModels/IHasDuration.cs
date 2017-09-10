using System;
using System.Drawing;

namespace Cobalt.Common.UI.ViewModels
{
    public interface IHasDuration
    {
        TimeSpan Duration { get; set; }
    }

    public class HasDuration : IHasDuration
    {
        private Func<TimeSpan> _getDur;
        private Action<TimeSpan> _setDur;

        public TimeSpan Duration
        {
            get => _getDur();
            set => _setDur(value);
        }

        public static IHasDuration From(Func<TimeSpan> get, Action<TimeSpan> set)
        {
            return new HasDuration
            {
                _getDur = get,
                _setDur = set
            };
        }
    }
}