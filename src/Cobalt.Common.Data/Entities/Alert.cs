using System.ComponentModel;
using System.ComponentModel.DataAnnotations.Schema;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.Data.Entities;

// Watch for inheritance support soon. Then make this record abstract & remove the properties
// ref: https://github.com/dotnet/efcore/issues/31250
/// <summary>
///     Action to take once the <see cref="Alert.UsageLimit" /> has been reached.
/// </summary>
[ComplexType]
public record TriggerAction(long Tag, string? MessageContent = null, TimeSpan? DimDuration = null)
{
    public const long KillTag = 0;
    public const long MessageTag = 1;
    public const long DimTag = 2;

    public sealed record Kill() : TriggerAction(0);

    public sealed record Message(string Content) : TriggerAction(1, Content);

    public sealed record Dim(TimeSpan Duration) : TriggerAction(2, DimDuration: Duration);
}

/// <summary>
///     How long the monitoring duration should be for an <see cref="Alert" />
/// </summary>
public enum TimeFrame
{
    Daily,
    Weekly,
    Monthly
}

/// <summary>
///     Monitoring record describing an usage limit for how long you use an <see cref="Cobalt.Common.Data.Entities.App" />
///     or a collection of Apps under a <see cref="Tag" />, the actions to take when that limit is reached, and the
///     reminders.
/// </summary>
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

    [DefaultValue(1)] public long Version { get; set; }

    // can't autoincrement on integer partial keys, so use random guid instead
    public required Guid Guid { get; set; }

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