using System.ComponentModel.DataAnnotations.Schema;

namespace Cobalt.Common.Data.Entities;

[Table("interaction_period")]
public class InteractionPeriod : IEntity, IHasDuration
{
    [Column("start")] public long StartTicks { get; set; }

    [Column("end")] public long EndTicks { get; set; }

    [NotMapped] public DateTime Start => new(StartTicks);
    [NotMapped] public DateTime End => new(EndTicks);

    [Column("mouseclicks")] public long MouseClicks { get; set; }

    [Column("keystrokes")] public long KeyStrokes { get; set; }
    public long Id { get; set; }
    [NotMapped] public TimeSpan Duration => End - Start;
}