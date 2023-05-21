using System.ComponentModel.DataAnnotations.Schema;

namespace Cobalt.Common.Data.Entities;

[Table("usage")]
public class Usage
{
    public long Id { get; set; }

    [ForeignKey("session")] public Session Session { get; set; }

    public DateTime Start { get; set; }
    public DateTime End { get; set; }
}