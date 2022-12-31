using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;

namespace Cobalt.Common.Data.Models;

[Table("tag")]
public class Tag
{
    [Required] public long Id { get; set; } = default!;
    [Required] public string Name { get; set; } = default!;
    [Required] public string? Color { get; set; } = default!;
    public List<App> Apps { get; set; } = default!;
}