using System.ComponentModel.DataAnnotations.Schema;

namespace Cobalt.Common.Data.Models;

[Table("session")]
public class Session
{
    public long Id { get; set; } = default!;

    [ForeignKey("app")] public App App { get; set; } = default!;

    public string Title { get; set; } = default!;

    [Column("cmd_line")] public string? CommandLine { get; set; } = default!;

    public List<Usage> Usages { get; set; } = default!;
}