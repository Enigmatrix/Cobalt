using System.ComponentModel;
using System.ComponentModel.DataAnnotations;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.Data.Entities;

/// <summary>
///     Notifications to send upon a certain threshold of an <see cref="Alert.UsageLimit" /> being reached.
/// </summary>
[PrimaryKey(nameof(Id), nameof(Version))]
public class Reminder : IEntity
{
    public required Alert Alert { get; set; }

    [Range(0.0, 1.0)] public double Threshold { get; set; }

    public required string Message { get; set; }
    public List<ReminderEvent> ReminderEvents { get; } = new();

    [DefaultValue(1)] public long Version { get; set; }

    public long Id { get; set; }

    long IEntity.Id => HashCode.Combine(Id, Version);

    public Reminder Clone()
    {
        return new Reminder
        {
            Alert = Alert,
            Threshold = Threshold,
            Message = Message,
            Version = Version,
            Id = Id
        };
    }
}