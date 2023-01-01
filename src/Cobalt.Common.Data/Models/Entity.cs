using System.ComponentModel.DataAnnotations;

namespace Cobalt.Common.Data.Models;
public abstract class Entity
{
    [Required] public long Id { get; set; } = default!;
}
