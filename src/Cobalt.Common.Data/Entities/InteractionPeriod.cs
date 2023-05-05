namespace Cobalt.Common.Data.Entities;

public class InteractionPeriod
{
    public long Id { get; set; }
    public DateTime Start { get; set; }
    public DateTime End { get; set; }
    public long MouseClicks { get; set; }
    public long KeyStrokes { get; set; }
}
