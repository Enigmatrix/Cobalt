using System;

namespace Cobalt.Common.Data.Entities
{
    public class Reminder : Entity
    {
        public TimeSpan Offset { get; set; }
        public ReminderAction Action { get; set; }
    }

    public abstract class ReminderAction
    {
    }

    public class WarnReminderAction : ReminderAction
    {
    }

    public class ScriptReminderAction : ReminderAction
    {
        public string Script { get; set; }
    }

    public class CustomWarnReminderAction : ReminderAction
    {
        public string Warning { get; set; }
    }
}