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
    private readonly long _actionTag = default!;
    private readonly bool _targetIsApp = default!;
    private long? _actionInt0 = default!;
    private string? _actionText0 = default!;
    private App? _app = default!;
    private Tag? _tag = default!;

    public long Id { get; set; }

    public Target Target =>
        _targetIsApp switch
        {
            true => new Target.AppTarget(_app ?? throw new ArgumentNullException(nameof(_app))),
            false => new Target.TagTarget(_tag ?? throw new ArgumentNullException(nameof(_tag)))
        };

    [Column("usage_limit")] public long UsageLimitTicks { get; set; }

    [NotMapped] public TimeSpan UsageLimit => TimeSpan.FromTicks(UsageLimitTicks);

    [Column("time_frame")] public TimeFrame TimeFrame { get; set; }

    public Action Action =>
        _actionTag switch
        {
            0 => new Action.Kill(),
            1 => new Action.Dim(
                TimeSpan.FromTicks(_actionInt0 ?? throw new ArgumentNullException(nameof(_actionInt0)))),
            2 => new Action.Message(_actionText0 ?? throw new ArgumentNullException(nameof(_actionText0))),
            _ => throw new InvalidOperationException() // TODO replace with DU Exception
        };

    public List<Reminder> Reminders { get; set; }
}