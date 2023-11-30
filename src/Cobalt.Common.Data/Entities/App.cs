using System.ComponentModel.DataAnnotations.Schema;

namespace Cobalt.Common.Data.Entities;

// Watch for inheritance support soon. Then make this record abstract & remove the properties
// ref: https://github.com/dotnet/efcore/issues/31250
[ComplexType]
public record AppIdentity(bool IsWin32, string PathOrAumid)
{
    public sealed record Win32(string Path) : AppIdentity(true, Path);

    public sealed record UWP(string Aumid) : AppIdentity(true, Aumid);
}

public class App : IEntity
{
    // This property is used to check if the App details have been finalized. If not,
    // all fields except Id, Identity and Initialized will be set to empty values.
    public bool Initialized { get; set; }
    public required string Name { get; set; }
    public required string Description { get; set; }
    public required string Company { get; set; }
    public required string Color { get; set; }
    public required AppIdentity Identity { get; set; }

    public List<Tag> Tags { get; } = new();

    public long Id { get; set; }
    // Blob
}