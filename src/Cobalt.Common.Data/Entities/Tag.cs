using System.ComponentModel.DataAnnotations.Schema;

namespace Cobalt.Common.Data.Entities;

[Table("tag")]
public class Tag : IEntity, IHasName, IHasColor
{
    public List<App> Apps { get; set; }
    public long Id { get; set; }
    public string Color { get; set; }
    public string Name { get; set; }
}