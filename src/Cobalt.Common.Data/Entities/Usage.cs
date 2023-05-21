using System.ComponentModel.DataAnnotations.Schema;

namespace Cobalt.Common.Data.Entities;

[Table("usage")]
public class Usage
{
    public long Id { get; set; }

    [ForeignKey("session")] public Session Session { get; set; }

    [Column("start")] public long StartTicks { get; set; }

    [Column("end")] public long EndTicks { get; set; }

    [NotMapped] public DateTime Start => new(StartTicks);
    [NotMapped] public DateTime End => new(EndTicks);
}