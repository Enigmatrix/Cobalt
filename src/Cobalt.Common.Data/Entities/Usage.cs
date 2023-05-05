namespace Cobalt.Common.Data.Entities;

public class Usage
{
    public long Id { get; set; }
    public Session Session { get; set; }
    public DateTime Start { get; set; }
    public DateTime End { get; set; }
}
