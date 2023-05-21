using System.ComponentModel.DataAnnotations.Schema;

namespace Cobalt.Common.Data.Entities;

[Table("reminder")]
public class Reminder
{
    public long Id { get; set; }
    [ForeignKey("alert")]
    public Alert Alert { get; set; }
    public double Threshold { get; set; }
    public string Message { get; set; }
}
