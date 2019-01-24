using System;

namespace Cobalt.Common.Data.Entities
{
    public abstract class Alert : Entity
    {
        public TimeSpan MaxDuration { get; set; }
        public bool Enabled { get; set; }
        public RunAction Action { get; set; }
        public TimeRange TimeRange { get; set; }
        public AppUsageType UsageType { get; set; }
    }

    public class AppAlert : Alert
    {
        public App App { get; set; }
    }

    public class TagAlert : Alert
    {
        public Tag Tag { get; set; }
    }

    public abstract class RunAction
    {
    }

    public class MessageRunAction : RunAction
    {
    }

    public class KillRunAction : RunAction
    {
    }

    public class CustomMessageRunAction : RunAction
    {
        public string Message { get; set; }
    }

    public class ScriptMessageRunAction : RunAction
    {
        public string Script { get; set; }
    }

    public abstract class TimeRange
    {
    }

    public class OnceTimeRange : TimeRange
    {
        public DateTime Start { get; set; }
        public DateTime End { get; set; }
    }

    public class RepeatingTimeRange : TimeRange
    {
        public RepeatingTimeRangeType Type { get; set; }
    }

    public enum RepeatingTimeRangeType : long
    {
        Daily = 0L,
        Weekly = 1L,
        Monthly = 2L
    }
}