using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;
using Cobalt.Common.Utils;

namespace Cobalt.Common.Data.Models;

[Table("app")]
public class App : Entity
{
    // TODO maybe can internal props? for query purposes
    [Required] [Column("identity_tag")] private int _identityTag;

    [Column("identity_text0")] private string _identityText0 = default!;

    [Required] public string Name { get; set; } = default!;
    [Required] public string Description { get; set; } = default!;
    [Required] public string Company { get; set; } = default!;
    public string? Color { get; set; } = default!;

    // Using this property in a query is a bad idea since it will materialize the tabs too early
    [NotMapped]
    public AppIdentity Identity
    {
        get =>
            _identityTag switch
            {
                0 => new AppIdentity.Win32(_identityText0),
                1 => new AppIdentity.Uwp(_identityText0),
                _ => throw new ToDiscriminatedUnionException<AppIdentity>(_identityTag)
            };
        set
        {
            switch (value)
            {
                case AppIdentity.Win32 win32:
                {
                    _identityTag = 0;
                    _identityText0 = win32.Path;
                    break;
                }
                case AppIdentity.Uwp uwp:
                {
                    _identityTag = 1;
                    _identityText0 = uwp.Aumid;
                    break;
                }
                default:
                    throw new FromDiscriminatedUnionException<AppIdentity>();
            }
        }
    }

    public List<Session> Sessions { get; set; } = default!;
    public List<Tag> Tags { get; set; } = default!;

    // TODO icon
}

public abstract record AppIdentity
{
    public sealed record Win32(string Path) : AppIdentity;

    public sealed record Uwp(string Aumid) : AppIdentity;
}