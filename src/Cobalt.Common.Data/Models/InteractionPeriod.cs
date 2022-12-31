using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;

namespace Cobalt.Common.Data.Models;

[Table("interaction_period")]
public class InteractionPeriod
{
    [Required] public long Id { get; set; } = default!;
    [Required] public int MouseClicks { get; set; }
    [Required] public int KeyStrokes { get; set; }
    [Required] public DateTime Start { get; set; } = default!;
    [Required] public DateTime End { get; set; } = default!;
}