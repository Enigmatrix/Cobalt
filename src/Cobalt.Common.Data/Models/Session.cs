using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;

namespace Cobalt.Common.Data.Models;

[Table("session")]
public class Session
{
    [Required] public long Id { get; set; } = default!;

    [Required] [ForeignKey("app")] public App App { get; set; } = default!;

    [Required] public string Title { get; set; } = default!;

    [Column("cmd_line")] public string? CommandLine { get; set; } = default!;

    public List<Usage> Usages { get; set; } = default!;
}