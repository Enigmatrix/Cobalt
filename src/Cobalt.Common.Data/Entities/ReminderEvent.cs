namespace Cobalt.Common.Data.Entities;

public class ReminderEvent : IEntity
{
    public required Reminder Reminder { get; set; }
    public required DateTime Timestamp { get; set; }
    public long Id { get; set; }
}