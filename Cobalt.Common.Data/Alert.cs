using System;

namespace Cobalt.Common.Data
{
    public abstract class Alert : Entity
    {
        public TimeSpan MaxDuration { get; set; }
        public AlertAction AlertAction { get; set; }
        public TimeSpan ReminderOffset { get; set; }
        public bool IsEnabled { get; set; }
        public AlertRange Range { get; set; }
    }
}