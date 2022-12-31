using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;

namespace Cobalt.Common.Data.Models;

[Table("app")]
public class App
{
    // maybe can internal props? for query purposes
    [Required] [Column("identity_tag")] private readonly int _identityTag = default!;

    [Column("identity_text0")] private readonly string _identityText0 = default!;

    [Required] public long Id { get; set; } = default!;
    [Required] public string Name { get; set; } = default!;
    [Required] public string Description { get; set; } = default!;
    [Required] public string Company { get; set; } = default!;
    public string? Color { get; set; } = default!;

    // Using this property in a query is a bad idea since it will materialize the tabs too early
    [NotMapped]
    public AppIdentity Identity => _identityTag switch
    {
        0 => new AppIdentity.Win32(_identityText0),
        1 => new AppIdentity.Uwp(_identityText0),
        _ => throw new Exception(
            $"Discriminated Union (AppIdentity) does not contain tag={_identityTag}") // TODO create exception for this in Utils
    };

    public List<Session> Sessions { get; set; } = default!;
    public List<Tag> Tags { get; set; } = default!;

    // TODO icon
}

public abstract record AppIdentity
{
    public sealed record Win32(string Path) : AppIdentity;

    public sealed record Uwp(string Aumid) : AppIdentity;
}