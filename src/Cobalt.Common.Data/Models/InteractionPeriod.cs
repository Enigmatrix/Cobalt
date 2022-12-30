using System.ComponentModel.DataAnnotations.Schema;

namespace Cobalt.Common.Data.Models;

[Table("interaction_period")]
public class InteractionPeriod
{
    public long Id { get; set; } = default!;
    public int MouseClicks { get; set; }
    public int KeyStrokes { get; set; }
    public DateTime Start { get; set; } = default!;
    public DateTime End { get; set; } = default!;
}