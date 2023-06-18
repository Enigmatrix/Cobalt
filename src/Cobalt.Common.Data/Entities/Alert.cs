using System.ComponentModel.DataAnnotations.Schema;
using Cobalt.Common.Utils;

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
public class Alert : IEntity
{
    private long? _actionInt0;
    private long _actionTag;
    private string? _actionText0;
    private App? _app;
    private Tag? _tag;
    private bool _targetIsApp;

    [NotMapped]
    public Target Target
    {
        get =>
            _targetIsApp switch
            {
                true => new Target.AppTarget(_app ?? throw new ArgumentNullException(nameof(_app))),
                false => new Target.TagTarget(_tag ?? throw new ArgumentNullException(nameof(_tag)))
            };
        set
        {
            switch (value)
            {
                case Target.AppTarget app:
                    _targetIsApp = true;
                    _app = app.App;
                    break;
                case Target.TagTarget tag:
                    _targetIsApp = true;
                    _tag = tag.Tag;
                    break;
                default:
                    throw new DiscriminatedUnionException<Target>(nameof(Target));
            }
        }
    }

    [Column("usage_limit")] public long UsageLimitTicks { get; set; }

    [NotMapped]
    public TimeSpan UsageLimit
    {
        get => TimeSpan.FromTicks(UsageLimitTicks);
        set => UsageLimitTicks = value.Ticks;
    }

    [Column("time_frame")] public TimeFrame TimeFrame { get; set; }

    [NotMapped]
    public Action Action
    {
        get =>
            _actionTag switch
            {
                0 => new Action.Kill(),
                1 => new Action.Dim(
                    TimeSpan.FromTicks(_actionInt0 ?? throw new ArgumentNullException(nameof(_actionInt0)))),
                2 => new Action.Message(_actionText0 ?? throw new ArgumentNullException(nameof(_actionText0))),
                _ => throw new DiscriminatedUnionException<Action>(nameof(Action))
            };
        set
        {
            switch (value)
            {
                case Action.Kill kill:
                    _actionTag = 0;
                    break;
                case Action.Dim dim:
                    _actionTag = 1;
                    _actionInt0 = dim.Duration.Ticks;
                    break;
                case Action.Message message:
                    _actionTag = 2;
                    _actionText0 = message.Text;
                    break;
                default:
                    throw new DiscriminatedUnionException<Action>(nameof(Action));
            }
        }
    }

    public List<Reminder> Reminders { get; set; } = default!;

    public long Id { get; set; }
}