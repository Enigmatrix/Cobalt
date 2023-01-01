using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;

namespace Cobalt.Common.Data.Entities;

[Table("tag")]
public class Tag : Entity
{
    [Required] public string Name { get; set; } = default!;
    [Required] public string? Color { get; set; } = default!;
    public List<App> Apps { get; set; } = default!;
}