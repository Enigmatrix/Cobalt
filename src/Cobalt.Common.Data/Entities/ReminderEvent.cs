namespace Cobalt.Common.Data.Entities;

/// <summary>
///     An instance of <see cref="Reminder" /> triggering.
/// </summary>
public class ReminderEvent : IEntity
{
    public required Reminder Reminder { get; set; }
    public required DateTime Timestamp { get; set; }
    public long Id { get; set; }
}