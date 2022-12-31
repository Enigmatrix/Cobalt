using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;
using Cobalt.Common.Utils;

namespace Cobalt.Common.Data.Models;

public enum TimeFrame
{
    Daily,
    Weekly,
    Monthly
}

public abstract record ExceedAction
{
    public sealed record Kill : ExceedAction;

    public sealed record Message(string Content) : ExceedAction;
}

[Table("alert")]
public class Alert
{
    // maybe can internal props? for query purposes
    [Required] [Column("action_tag")] private int _exceedActionTag;
    [Column("action_text0")] private string? _exceedActionText0;

    [Required] public long Id { get; set; } = default!;
    [Required] [Column("target_is_app")] public bool TargetIsApp { get; set; } = default!;
    [ForeignKey("app")] public App? App { get; set; } = default!;
    [ForeignKey("tag")] public Tag? Tag { get; set; } = default!;

    [Required] [Column("usage_limit")] internal long UsageLimitTicks { get; set; }

    [NotMapped]
    public TimeSpan UsageLimit
    {
        get => TimeSpan.FromTicks(UsageLimitTicks);
        set => UsageLimitTicks = value.Ticks;
    }

    [Required] [Column("time_frame")] public TimeFrame TimeFrame { get; set; } = default!;

    // Using this property in a query is a bad idea since it will materialize the tabs too early
    [NotMapped]
    public ExceedAction ExceedAction
    {
        get =>
            _exceedActionTag switch
            {
                0 => new ExceedAction.Kill(),
                1 => new ExceedAction.Message(_exceedActionText0!),
                _ => throw new ToDiscriminatedUnionException<ExceedAction>(_exceedActionTag)
            };
        set
        {
            switch (value)
            {
                case ExceedAction.Kill:
                {
                    _exceedActionTag = 0;
                    _exceedActionText0 = null;
                    break;
                }
                case ExceedAction.Message msg:
                {
                    _exceedActionTag = 1;
                    _exceedActionText0 = msg.Content;
                    break;
                }
                default:
                    throw new FromDiscriminatedUnionException<ExceedAction>();
            }
        }
    }
}