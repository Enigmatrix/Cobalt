using System.ComponentModel.DataAnnotations;

namespace Cobalt.Common.Data.Entities;

public abstract class Entity
{
    [Required] public long Id { get; set; } = default!;
}