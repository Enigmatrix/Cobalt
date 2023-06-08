using System.ComponentModel.DataAnnotations.Schema;

namespace Cobalt.Common.Data.Entities;

[Table("usage")]
public class Usage : IEntity, IHasDuration
{
    [ForeignKey("session")] public Session Session { get; set; } = default!;

    [Column("start")] public long StartTicks { get; set; } = default!;

    [Column("end")] public long EndTicks { get; set; } = default!;

    [NotMapped] public DateTime Start => new(StartTicks);
    [NotMapped] public DateTime End => new(EndTicks);
    public long Id { get; set; } = default!;
    [NotMapped] public TimeSpan Duration => End - Start;
}