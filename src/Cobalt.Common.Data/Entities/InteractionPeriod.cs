using System.ComponentModel.DataAnnotations.Schema;

namespace Cobalt.Common.Data.Entities;

[Table("interaction_period")]
public class InteractionPeriod
{
    public long Id { get; set; }
    public DateTime Start { get; set; }
    public DateTime End { get; set; }

    [Column("mouseclicks")]
    public long MouseClicks { get; set; }
    [Column("keystrokes")]
    public long KeyStrokes { get; set; }
}
