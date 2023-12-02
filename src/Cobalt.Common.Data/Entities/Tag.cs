namespace Cobalt.Common.Data.Entities;

public class Tag : IEntity, IHasName, IHasColor
{
    public List<App> Apps { get; } = new();
    public long Id { get; set; }
    public required string Color { get; set; }
    public required string Name { get; set; }
}