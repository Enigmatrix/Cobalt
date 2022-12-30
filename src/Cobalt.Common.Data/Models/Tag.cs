using System.ComponentModel.DataAnnotations.Schema;

namespace Cobalt.Common.Data.Models;

[Table("tag")]
public class Tag
{
    public long Id { get; set; } = default!;
    public string Name { get; set; } = default!;
    public string? Color { get; set; } = default!;
    public List<App> Apps { get; set; } = default!;
}