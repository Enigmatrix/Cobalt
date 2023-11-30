using System.ComponentModel.DataAnnotations;

namespace Cobalt.Common.Data.Entities;

public class Reminder
{
    public long Id { get; set; }
    public required Alert Alert { get; set; }

    [Range(0.0, 1.0)] public double Threshold { get; set; }

    public required string Message { get; set; }
    public List<ReminderEvent> ReminderEvents { get; } = new();
}