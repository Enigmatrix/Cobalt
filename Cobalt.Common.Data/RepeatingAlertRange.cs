using System;

namespace Cobalt.Common.Data
{
    public class RepeatingAlertRange : AlertRange
    {
        public TimeSpan DailyStartOffset { get; set; }
        public TimeSpan DailyEndOffset { get; set; }
        public RepeatType RepeatType { get; set; }
    }
}