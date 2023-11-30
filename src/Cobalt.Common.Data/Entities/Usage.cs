namespace Cobalt.Common.Data.Entities;

public class Usage : IEntity
{
    public required Session Session { get; set; }
    public required DateTime Start { get; set; }
    public required DateTime End { get; set; }
    public long Id { get; set; }
}