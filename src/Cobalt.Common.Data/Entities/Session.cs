using System.ComponentModel.DataAnnotations.Schema;

namespace Cobalt.Common.Data.Entities;

[Table("session")]
public class Session
{
    public long Id { get; set; }
    [ForeignKey("app")]
    public App App { get; set; }
    public string Title { get; set; }
    [Column("cmd_line")]
    public string? CmdLine { get; set; }
}
