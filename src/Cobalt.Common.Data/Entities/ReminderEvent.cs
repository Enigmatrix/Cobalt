namespace Cobalt.Common.Data.Entities;

public class ReminderEvent
{
    public long Id { get; set; }
    public required Reminder Reminder { get; set; }
    public required DateTime Timestamp { get; set; }
}