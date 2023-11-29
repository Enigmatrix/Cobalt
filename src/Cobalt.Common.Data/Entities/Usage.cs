namespace Cobalt.Common.Data.Entities;

public class Usage
{
    public long Id { get; set; }
    public required Session Session { get; set; }
    public required DateTime Start { get; set; }
    public required DateTime End { get; set; }
}