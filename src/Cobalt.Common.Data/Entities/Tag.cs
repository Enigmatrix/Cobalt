namespace Cobalt.Common.Data.Entities;

public class Tag
{
    public long Id { get; set; }
    public required string Name { get; set; }
    public required string Color { get; set; }
    public List<App> Apps { get; } = new();
}