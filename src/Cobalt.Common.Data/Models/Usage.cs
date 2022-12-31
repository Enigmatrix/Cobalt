using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;

namespace Cobalt.Common.Data.Models;

[Table("usage")]
public class Usage
{
    [Required] public long Id { get; set; } = default!;

    [Required] [ForeignKey("session")] public Session Session { get; set; } = default!;

    [Required] [Column("start")] internal long StartTicks { get; set; } = default!;
    [Required] [Column("end")] internal long EndTicks { get; set; } = default!;

    public DateTime Start => DateTime.FromFileTime(StartTicks);
    public DateTime End => DateTime.FromFileTime(EndTicks);
}