using System.ComponentModel.DataAnnotations.Schema;

namespace Cobalt.Common.Data.Entities;

public abstract record Target
{
    public sealed record AppTarget(App App) : Target;
    public sealed record TagTarget(Tag Tag) : Target;

}

public enum TimeFrame
{
    Daily,
    Weekly,
    Monthly
}

public abstract record Action
{
    public sealed record Kill : Action;
    public sealed record Dim(TimeSpan Duration) : Action;
    public sealed record Message(string Text) : Action;
}

[Table("alert")]
public class Alert
{
    public long Id { get; set; }
    public Target Target { get; set; }
    public TimeSpan UsageLimit { get; set; }
    public TimeFrame TimeFrame { get; set; }
    public Action Action { get; set; }
    public List<Reminder> Reminders { get; set; }
}
