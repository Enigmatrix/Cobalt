namespace Cobalt.Common.Data.Entities;

public class Tag : IEntity
{
    public required string Name { get; set; }
    public required string Color { get; set; }
    public List<App> Apps { get; } = new();
    public long Id { get; set; }
}