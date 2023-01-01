using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;

namespace Cobalt.Common.Data.Entities;

[Table("interaction_period")]
public class InteractionPeriod : Entity
{
    [Required] public int MouseClicks { get; set; }
    [Required] public int KeyStrokes { get; set; }

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