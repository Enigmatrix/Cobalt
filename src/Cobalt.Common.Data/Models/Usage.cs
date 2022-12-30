using System.ComponentModel.DataAnnotations.Schema;

namespace Cobalt.Common.Data.Models;

[Table("usage")]
public class Usage
{
    public long Id { get; set; } = default!;

    [ForeignKey("session")] public Session Session { get; set; } = default!;

    public DateTime Start { get; set; } = default!;
    public DateTime End { get; set; } = default!;
}