using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;

namespace Cobalt.Common.Data.Models;

[Table("usage")]
public class Usage
{
    [Required] public long Id { get; set; } = default!;

    [Required] [ForeignKey("session")] public Session Session { get; set; } = default!;

    [Required] [Column("start")] internal long StartTicks { get; set; }
    [Required] [Column("end")] internal long EndTicks { get; set; }

    [NotMapped]
    public DateTime Start
    {
        get => DateTime.FromFileTime(StartTicks);
        set => StartTicks = value.ToFileTime();
    }

    [NotMapped]
    public DateTime End
    {
        get => DateTime.FromFileTime(EndTicks);
        set => EndTicks = value.ToFileTime();
    }
}