using System.ComponentModel;
using System.ComponentModel.DataAnnotations.Schema;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.Data.Entities;

// Watch for inheritance support soon. Then make this record abstract & remove the properties
// ref: https://github.com/dotnet/efcore/issues/31250
[ComplexType]
public record TriggerAction(long Tag, string? MessageContent, TimeSpan? DimDuration)
{
    public sealed record Kill() : TriggerAction(0, null, null);

    public sealed record Message(string Content) : TriggerAction(1, Content, null);

    public sealed record Dim(TimeSpan Duration) : TriggerAction(2, null, Duration);
}

public enum TimeFrame
{
    Daily,
    Weekly,
    Monthly
}

[PrimaryKey(nameof(Guid), nameof(Version))]
public class Alert : IEntity
{
    public App? App { get; set; }
    public Tag? Tag { get; set; }
    public TimeSpan UsageLimit { get; set; }
    public TimeFrame TimeFrame { get; set; }
    public required TriggerAction TriggerAction { get; set; }
    public List<Reminder> Reminders { get; } = new();
    public List<AlertEvent> AlertEvents { get; } = new();

    [DefaultValue(1)]
    public long Version { get; set; }

    // can't autoincrement on integer partial keys, so use random guid instead
    public Guid Guid { get; set; } = Guid.NewGuid();

    public long Id => HashCode.Combine(Guid, Version);

    public Alert Clone()
    {
        return new Alert
        {
            App = App,
            Tag = Tag,
            Guid = Guid,
            UsageLimit = UsageLimit,
            TimeFrame = TimeFrame,
            TriggerAction = TriggerAction,
            Version = Version
        };
    }
}