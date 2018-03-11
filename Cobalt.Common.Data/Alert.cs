using System;

namespace Cobalt.Common.Data
{
    public enum AlertAction
    {
        Message,
        Kill
    }

    public enum RepeatType
    {
        Daily,
        Weekly,
        Weekday,
        Weekend,
        Monthly
    }

    public abstract class Alert : Entity
    {
        public TimeSpan MaxDuration { get; set; }
        public AlertAction AlertAction { get; set; }
        public TimeSpan ReminderOffset { get; set; }
        public bool IsEnabled { get; set; }
        public AlertRange Range { get; set; }
    }

    public class AppAlert : Alert
    {
        public App App { get; set; }
    }

    public class TagAlert : Alert
    {
        public Tag Tag { get; set; }
    }

    public abstract class AlertRange
    {
    }

    public class OnceAlertRange : AlertRange
    {
        public DateTime StartTimestamp { get; set; }
        public DateTime EndTimestamp { get; set; }
    }

    public class RepeatingAlertRange : AlertRange
    {
        public TimeSpan DailyStartOffset { get; set; }
        public TimeSpan DailyEndOffset { get; set; }
        public RepeatType RepeatType { get; set; }
    }
}