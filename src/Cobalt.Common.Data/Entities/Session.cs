using System.ComponentModel.DataAnnotations.Schema;

namespace Cobalt.Common.Data.Entities;

[Table("session")]
public class Session : IEntity
{
    [ForeignKey("app")] public App App { get; set; } = default!;

    public string Title { get; set; } = default!;

    [Column("cmd_line")] public string? CmdLine { get; set; } = default!;

    public List<Usage> Usages { get; set; } = default!;
    public long Id { get; set; } = default!;
}