using System.ComponentModel.DataAnnotations.Schema;

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
    [Column("action_tag")] private int _exceedActionTag;
    [Column("action_text0")] private string? _exceedActionText0;

    public long Id { get; set; } = default!;
    [ForeignKey("app")] public App? App { get; set; } = default!;
    [ForeignKey("tag")] public Tag? Tag { get; set; } = default!;

    [Column("usage_limit")] public TimeSpan UsageLimit { get; set; } = default!;

    [Column("time_frame")] public TimeFrame TimeFrame { get; set; } = default!;

    // Using this property in a query is a bad idea since it will materialize the tabs too early
    [NotMapped] public ExceedAction ExceedAction
    {
        get =>
            _exceedActionTag switch
            {
                0 => new ExceedAction.Kill(),
                1 => new ExceedAction.Message(_exceedActionText0!),
                _ => throw new Exception(
                    $"Discriminated Union (ExceedAction) does not contain tag={_exceedActionTag}") // TODO create exception for this in Utils
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
                    throw new Exception("Discriminated Union (ExceedAction) with unknown branch");
            }
        }
    }
}