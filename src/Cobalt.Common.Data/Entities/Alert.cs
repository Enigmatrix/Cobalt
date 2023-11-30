using System.ComponentModel.DataAnnotations.Schema;

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

public class Alert : IEntity
{
    public App? App { get; set; }
    public Tag? Tag { get; set; }
    public TimeSpan UsageLimit { get; set; }
    public TimeFrame TimeFrame { get; set; }
    public required TriggerAction TriggerAction { get; set; }
    public List<Reminder> Reminders { get; } = new();
    public List<AlertEvent> AlertEvents { get; } = new();
    public long Id { get; set; }
}