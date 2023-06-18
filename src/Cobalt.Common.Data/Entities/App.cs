using System.ComponentModel.DataAnnotations.Schema;
using Cobalt.Common.Utils;

namespace Cobalt.Common.Data.Entities;

public abstract record AppIdentity
{
    public sealed record Win32(string Path) : AppIdentity;

    public sealed record Uwp(string Aumid) : AppIdentity;
}

[Table("app")]
public class App : IEntity, IHasName, IHasColor
{
    private long _identityTag;
    private string _identityText0 = default!;
    public string Description { get; set; } = default!;
    public string Company { get; set; } = default!;

    [NotMapped]
    public AppIdentity Identity
    {
        get =>
            _identityTag switch
            {
                0 => new AppIdentity.Win32(_identityText0),
                1 => new AppIdentity.Uwp(_identityText0),
                _ => throw new DiscriminatedUnionException<AppIdentity>(nameof(Identity))
            };
        set
        {
            switch (value)
            {
                case AppIdentity.Win32 win32:
                    _identityTag = 0;
                    _identityText0 = win32.Path;
                    break;
                case AppIdentity.Uwp uwp:
                    _identityTag = 1;
                    _identityText0 = uwp.Aumid;
                    break;
                default:
                    throw new DiscriminatedUnionException<AppIdentity>(nameof(Identity));
            }
        }
    }

    public List<Tag> Tags { get; set; } = default!;
    public List<Session> Sessions { get; set; } = default!;

    public long Id { get; set; } = default!;
    public string Color { get; set; } = default!;
    public string Name { get; set; } = default!;
}