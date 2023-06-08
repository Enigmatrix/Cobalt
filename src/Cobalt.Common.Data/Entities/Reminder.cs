using System.ComponentModel.DataAnnotations.Schema;

namespace Cobalt.Common.Data.Entities;

[Table("reminder")]
public class Reminder : IEntity
{
    [ForeignKey("alert")] public Alert Alert { get; set; } = default!;

    public double Threshold { get; set; } = default!;
    public string Message { get; set; } = default!;
    public long Id { get; set; } = default!;
}