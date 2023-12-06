namespace Cobalt.Common.Data.Entities;

/// <summary>
///     Collection of <see cref="App" /> with a <see cref="Name" />
/// </summary>
public class Tag : IEntity, IHasName, IHasColor
{
    public List<App> Apps { get; } = new();
    public long Id { get; set; }
    public required string Color { get; set; }
    public required string Name { get; set; }
}