using System.ComponentModel.DataAnnotations.Schema;

namespace Cobalt.Common.Data.Entities;

public abstract record AppIdentity
{
    public sealed record Win32(string Path) : AppIdentity;

    public sealed record Uwp(string Aumid) : AppIdentity;
}

[Table("app")]
public class App : IEntity
{
    private long _identityTag = default!;
    private string _identityText0 = default!;

    public long Id { get; set; }
    public string Name { get; set; }
    public string Description { get; set; }
    public string Company { get; set; }
    public string Color { get; set; }

    public AppIdentity Identity
    {
        get =>
            _identityTag switch
            {
                0 => new AppIdentity.Win32(_identityText0),
                1 => new AppIdentity.Uwp(_identityText0),
                _ => throw new InvalidOperationException() // TODO throw custom exception
            };
        set {
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
                    throw new InvalidOperationException(); // TODO throw custom exception
            }
        }
    }

    // Icon is not represented here

    public List<Tag> Tags { get; set; }
    public List<Session> Sessions { get; set; } = default!;
}