namespace Cobalt.Common.Data.Entities;

public class Session : IEntity
{
    public required string Title { get; set; }
    public required App App { get; set; }
    public List<Usage> Usages { get; } = new();
    public long Id { get; set; }
}