using System.ComponentModel.DataAnnotations.Schema;

namespace Cobalt.Common.Data.Entities;

[Table("tag")]
public class Tag : IEntity, IHasName, IHasColor
{
    public List<App> Apps { get; set; } = default!;
    public long Id { get; set; } = default!;
    public string Color { get; set; } = default!;
    public string Name { get; set; } = default!;
}