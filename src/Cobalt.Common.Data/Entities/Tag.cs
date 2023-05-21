using System.ComponentModel.DataAnnotations.Schema;

namespace Cobalt.Common.Data.Entities;

[Table("tag")]
public class Tag
{
    public long Id { get; set; }
    public string Name { get; set; }
    public string Color { get; set; }
}
